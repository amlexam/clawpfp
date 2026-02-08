use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::keypair::Keypair;
use sqlx::PgPool;
use crate::config::Config;
use crate::services::tree_manager::TreeManager;

pub struct AppState {
    pub config: Config,
    pub rpc_client: Arc<RpcClient>,
    pub payer: Arc<Keypair>,
    pub db: PgPool,
    pub tree_manager: TreeManager,
    pub http_client: reqwest::Client,
}
