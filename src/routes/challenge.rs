use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use crate::error::AppError;
use crate::models::challenge::ChallengeResponse;
use crate::services;
use crate::state::AppState;
use crate::db;

pub async fn challenge_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ChallengeResponse>, AppError> {
    let challenge = services::challenge::generate_challenge(state.config.challenge_expiry_seconds);

    db::challenges::insert_challenge(&state.db, &challenge).await?;

    let response = ChallengeResponse::from(&challenge);
    Ok(Json(response))
}
