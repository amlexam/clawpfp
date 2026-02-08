use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use crate::state::AppState;
use crate::db;

pub async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let capacity_remaining = db::trees::get_tree_capacity_remaining(&state.db)
        .await
        .unwrap_or(0);
    let total_minted = db::mints::get_total_minted(&state.db)
        .await
        .unwrap_or(0);
    let active_tree = db::trees::get_active_tree(&state.db)
        .await
        .ok()
        .flatten()
        .map(|t| t.address)
        .unwrap_or_default();

    Json(json!({
        "status": "ok",
        "active_tree": active_tree,
        "tree_capacity_remaining": capacity_remaining,
        "total_minted": total_minted,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
