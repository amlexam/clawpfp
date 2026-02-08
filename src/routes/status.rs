use std::sync::Arc;
use axum::extract::{Path, State};
use axum::Json;
use crate::db;
use crate::error::AppError;
use crate::models::mint::StatusResponse;
use crate::state::AppState;

pub async fn status_handler(
    State(state): State<Arc<AppState>>,
    Path(tx_signature): Path<String>,
) -> Result<Json<StatusResponse>, AppError> {
    let mint_record = db::mints::get_mint_by_tx(&state.db, &tx_signature).await?;

    match mint_record {
        Some((tx_sig, status, asset_id, recipient, created_at)) => {
            Ok(Json(StatusResponse {
                tx_signature: tx_sig,
                status,
                asset_id: Some(asset_id),
                recipient: Some(recipient),
                confirmed_at: Some(created_at),
            }))
        }
        None => Err(AppError::NotFound("Transaction not found".into())),
    }
}
