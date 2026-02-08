use std::str::FromStr;
use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use crate::db;
use crate::error::AppError;
use crate::models::mint::{MintRequest, MintResponse};
use crate::services::{bubblegum, challenge, irys, metadata};
use crate::state::AppState;

pub async fn mint_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MintRequest>,
) -> Result<Json<MintResponse>, AppError> {
    // 1. Validate and consume challenge
    let ch = db::challenges::get_challenge(&state.db, &req.challenge_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Challenge not found".into()))?;

    if ch.status != "pending" {
        return Err(AppError::BadRequest("Challenge already used".into()));
    }
    if chrono::Utc::now() > ch.expires_at {
        db::challenges::expire_challenge(&state.db, &req.challenge_id).await?;
        return Err(AppError::Gone("Challenge expired".into()));
    }
    if !challenge::verify_challenge_answer(&ch, &req.answer) {
        return Err(AppError::BadRequest("Incorrect answer".into()));
    }

    // 2. Parse wallet address
    let leaf_owner = Pubkey::from_str(&req.wallet_address)
        .map_err(|_| AppError::BadRequest("Invalid wallet address".into()))?;

    // 3. Check collection mint is configured
    let collection_mint = state
        .config
        .collection_mint
        .ok_or_else(|| AppError::Internal("Collection mint not configured".into()))?;

    // 4. Get active tree (with rotation if needed)
    let tree_info = state.tree_manager.get_active_tree().await
        .map_err(|e| AppError::Internal(format!("Tree error: {}", e)))?;

    // 5. Generate metadata and upload to Arweave via Irys
    let mint_index = tree_info.current_leaf_index;
    let name = metadata::generate_name(&state.config, mint_index);
    let metadata_json = metadata::build_metadata_json(
        &name,
        &state.config.collection_symbol,
        &state.config.collection_description,
        &state.config.collection_image_url,
        state.config.seller_fee_basis_points,
        mint_index,
    );

    let uri = irys::upload(
        &state.http_client,
        metadata_json.as_bytes(),
        "application/json",
        &state.payer,
        &state.config.irys_node_url,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Metadata upload failed: {}", e)))?;

    tracing::info!("Metadata uploaded: {}", uri);

    // 6. Build mint instruction
    let mint_ix = bubblegum::build_mint_to_collection_ix(
        &state.payer.pubkey(),
        &tree_info.address,
        &leaf_owner,
        &collection_mint,
        name.clone(),
        state.config.collection_symbol.clone(),
        uri.clone(),
        state.config.seller_fee_basis_points,
    );

    // 7. Build, sign, send transaction
    let blockhash = state.rpc_client.get_latest_blockhash().await
        .map_err(|e| AppError::Internal(format!("RPC error: {}", e)))?;
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&state.payer.pubkey()),
        &[&*state.payer],
        blockhash,
    );
    let signature = state.rpc_client.send_and_confirm_transaction(&tx).await
        .map_err(|e| AppError::Internal(format!("Transaction failed: {}", e)))?;

    // 8. Derive asset ID
    let asset_id = bubblegum::derive_asset_id(&tree_info.address, mint_index);

    // 9. Record in database
    db::challenges::mark_challenge_consumed(&state.db, &req.challenge_id).await?;
    db::mints::insert_mint(
        &state.db,
        &asset_id.to_string(),
        &tree_info.address.to_string(),
        mint_index,
        &req.wallet_address,
        &uri,
        &name,
        &signature.to_string(),
        &req.challenge_id,
    )
    .await?;
    db::trees::increment_tree_leaf_index(&state.db, &tree_info.address.to_string()).await?;

    Ok(Json(MintResponse {
        success: true,
        tx_signature: signature.to_string(),
        asset_id: asset_id.to_string(),
        mint_index,
        message: "cNFT minted successfully".into(),
    }))
}
