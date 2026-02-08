use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub challenge_id: String,
    pub answer: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct MintResponse {
    pub success: bool,
    pub tx_signature: String,
    pub asset_id: String,
    pub mint_index: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub tx_signature: String,
    pub status: String,
    pub asset_id: Option<String>,
    pub recipient: Option<String>,
    pub confirmed_at: Option<String>,
}
