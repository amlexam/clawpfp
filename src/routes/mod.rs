use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};
use tower_http::services::ServeDir;
use crate::state::AppState;

pub mod health;
pub mod challenge;
pub mod mint;
pub mod status;

async fn serve_skill_md() -> Response {
    match tokio::fs::read_to_string("SKILL.md").await {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            content,
        ).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "SKILL.md not found").into_response(),
    }
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/challenge", get(challenge::challenge_handler))
        .route("/mint", post(mint::mint_handler))
        .route("/status/{tx_signature}", get(status::status_handler))
        .route("/skill.md", get(serve_skill_md))
        // Serve metadata JSON files at /metadata/{index}.json
        .nest_service("/metadata", ServeDir::new("metadata"))
        .with_state(state)
}
