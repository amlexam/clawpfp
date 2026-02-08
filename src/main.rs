use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use solana_sdk::signer::Signer;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;
mod setup;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cnft_mint_server=info".parse().unwrap()),
        )
        .init();

    let config = config::Config::from_env()?;

    // Load server keypair (supports JSON array [1,2,3,...] or base64 string)
    let key_bytes: Vec<u8> = {
        let raw = config.payer_keypair_json.trim();
        if raw.starts_with('[') {
            serde_json::from_str(raw)?
        } else {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, raw)
                .map_err(|e| anyhow::anyhow!("Invalid base64 keypair: {}", e))?
        }
    };
    let payer = Arc::new(
        solana_sdk::signer::keypair::Keypair::from_bytes(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid payer keypair: {}", e))?,
    );
    tracing::info!("Server pubkey: {}", payer.pubkey());

    // Initialize Solana RPC client
    let rpc_client = Arc::new(
        solana_client::nonblocking::rpc_client::RpcClient::new(config.solana_rpc_url.clone()),
    );

    // Check for subcommand
    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str());

    match subcommand {
        Some("setup") => {
            tracing::info!("Running collection setup...");
            let collection_mint = setup::setup_collection(
                &rpc_client,
                &payer,
                &config.collection_name,
                &config.collection_symbol,
                &config.base_metadata_uri,
            )
            .await?;

            println!("\n=== Collection Setup Complete ===");
            println!("Collection Mint: {}", collection_mint);
            println!("\nAdd this to your .env file:");
            println!("COLLECTION_MINT={}", collection_mint);
            return Ok(());
        }
        Some("serve") | None => {
            // Continue to server startup below
        }
        Some(other) => {
            eprintln!("Unknown subcommand: {}", other);
            eprintln!("Usage: cnft-mint-server [setup|serve]");
            std::process::exit(1);
        }
    }

    // ─── Server startup ───

    // Initialize PostgreSQL database
    // Disable prepared statement caching for PgBouncer (Supabase transaction pooler) compat
    let db_opts: sqlx::postgres::PgConnectOptions = config.database_url.parse::<sqlx::postgres::PgConnectOptions>()?
        .statement_cache_capacity(0);
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(db_opts)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    // Initialize tree manager
    let tree_manager = services::tree_manager::TreeManager::new(
        db.clone(),
        rpc_client.clone(),
        payer.clone(),
        config.clone(),
    );

    let http_client = reqwest::Client::new();

    let app_state = Arc::new(state::AppState {
        config: config.clone(),
        rpc_client,
        payer,
        db: db.clone(),
        tree_manager,
        http_client,
    });

    // Build router
    let app: Router = routes::create_router(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Spawn challenge cleanup task
    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let _ = sqlx::query(
                "UPDATE challenges SET status = 'expired'
                 WHERE status = 'pending' AND expires_at < NOW()",
            )
            .execute(&cleanup_db)
            .await;
        }
    });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
