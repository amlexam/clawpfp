use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Clone)]
pub struct Config {
    pub solana_rpc_url: String,
    pub payer_keypair_json: String,
    pub merkle_tree_max_depth: u32,
    pub merkle_tree_max_buffer_size: u32,
    pub merkle_tree_canopy_depth: u32,
    pub collection_mint: Option<Pubkey>,
    pub collection_name: String,
    pub collection_symbol: String,
    pub base_metadata_uri: String,
    pub seller_fee_basis_points: u16,
    pub collection_description: String,
    pub collection_image_url: String,
    pub irys_node_url: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
    pub challenge_expiry_seconds: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let collection_mint_str = std::env::var("COLLECTION_MINT").unwrap_or_default();
        let collection_mint = if collection_mint_str.is_empty() {
            None
        } else {
            Some(Pubkey::from_str(&collection_mint_str)?)
        };

        Ok(Config {
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
            payer_keypair_json: std::env::var("PAYER_KEYPAIR")
                .unwrap_or_else(|_| "[]".to_string()),
            merkle_tree_max_depth: std::env::var("MERKLE_TREE_MAX_DEPTH")
                .unwrap_or_else(|_| "14".to_string())
                .parse()?,
            merkle_tree_max_buffer_size: std::env::var("MERKLE_TREE_MAX_BUFFER_SIZE")
                .unwrap_or_else(|_| "64".to_string())
                .parse()?,
            merkle_tree_canopy_depth: std::env::var("MERKLE_TREE_CANOPY_DEPTH")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            collection_mint,
            collection_name: std::env::var("COLLECTION_NAME")
                .unwrap_or_else(|_| "MyCNFTCollection".to_string()),
            collection_symbol: std::env::var("COLLECTION_SYMBOL")
                .unwrap_or_else(|_| "CNFT".to_string()),
            base_metadata_uri: std::env::var("BASE_METADATA_URI")
                .unwrap_or_else(|_| "https://arweave.net/placeholder".to_string()),
            seller_fee_basis_points: std::env::var("SELLER_FEE_BASIS_POINTS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()?,
            collection_description: std::env::var("COLLECTION_DESCRIPTION")
                .unwrap_or_else(|_| "A compressed NFT minted by an AI agent.".to_string()),
            collection_image_url: std::env::var("COLLECTION_IMAGE_URL")
                .unwrap_or_else(|_| "https://placehold.co/500x500/6C5CE7/FFFFFF/png?text=cNFT".to_string()),
            irys_node_url: std::env::var("IRYS_NODE_URL")
                .unwrap_or_else(|_| "https://devnet.irys.xyz".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set (e.g. postgresql://user:pass@host:5432/dbname)"),
            rate_limit_per_second: std::env::var("RATE_LIMIT_PER_SECOND")
                .unwrap_or_else(|_| "2".to_string())
                .parse()?,
            rate_limit_burst: std::env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            challenge_expiry_seconds: std::env::var("CHALLENGE_EXPIRY_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
        })
    }
}
