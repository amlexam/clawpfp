use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use tower_http::services::ServeDir;
use crate::state::AppState;

pub mod health;
pub mod challenge;
pub mod mint;
pub mod status;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/challenge", get(challenge::challenge_handler))
        .route("/mint", post(mint::mint_handler))
        .route("/status/{tx_signature}", get(status::status_handler))
        .route("/skill.md", get(|| async {
            tokio::fs::read_to_string("skill.md").await.unwrap_or_default()
        }))
        // Serve metadata JSON files at /metadata/{index}.json
        .nest_service("/metadata", ServeDir::new("metadata"))
        .with_state(state)
}
