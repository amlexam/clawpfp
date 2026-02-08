use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::db;
use crate::models::tree::TreeInfo;
use crate::services::bubblegum;

pub struct TreeManager {
    pub db: SqlitePool,
    pub rpc_client: Arc<RpcClient>,
    pub payer: Arc<Keypair>,
    pub config: Config,
}

impl TreeManager {
    pub fn new(
        db: SqlitePool,
        rpc_client: Arc<RpcClient>,
        payer: Arc<Keypair>,
        config: Config,
    ) -> Self {
        TreeManager {
            db,
            rpc_client,
            payer,
            config,
        }
    }

    pub async fn get_active_tree(&self) -> anyhow::Result<TreeInfo> {
        let tree_row = db::trees::get_active_tree(&self.db).await?;
        match tree_row {
            Some(row) => {
                let info: TreeInfo = row.try_into()?;
                if info.current_leaf_index < info.max_capacity {
                    Ok(info)
                } else {
                    // Tree is full, create a new one
                    tracing::info!("Active tree is full, creating new tree");
                    db::trees::deactivate_tree(&self.db, &info.address.to_string()).await?;
                    self.create_and_register_tree().await
                }
            }
            None => {
                tracing::info!("No active tree found, creating initial tree");
                self.create_and_register_tree().await
            }
        }
    }

    pub async fn create_and_register_tree(&self) -> anyhow::Result<TreeInfo> {
        let max_depth = self.config.merkle_tree_max_depth;
        let max_buffer_size = self.config.merkle_tree_max_buffer_size;
        let canopy_depth = self.config.merkle_tree_canopy_depth;

        let (tree_pubkey, tx_sig) = self
            .create_merkle_tree(max_depth, max_buffer_size, canopy_depth)
            .await?;

        let max_capacity = 1u64 << max_depth;
        let collection_mint_str = self.config.collection_mint.map(|p| p.to_string());

        db::trees::insert_tree(
            &self.db,
            &tree_pubkey.to_string(),
            max_depth,
            max_buffer_size,
            canopy_depth,
            max_capacity,
            collection_mint_str.as_deref(),
            Some(&tx_sig),
        )
        .await?;

        Ok(TreeInfo {
            address: tree_pubkey,
            max_depth,
            max_buffer_size,
            canopy_depth,
            max_capacity,
            current_leaf_index: 0,
            is_active: true,
        })
    }

    async fn create_merkle_tree(
        &self,
        max_depth: u32,
        max_buffer_size: u32,
        canopy_depth: u32,
    ) -> anyhow::Result<(Pubkey, String)> {
        use mpl_bubblegum::instructions::CreateTreeConfigBuilder;

        let tree_keypair = Keypair::new();
        let bubblegum_id = bubblegum::bubblegum_program_id();

        // Derive Tree Config PDA
        let (tree_config, _) = Pubkey::find_program_address(
            &[tree_keypair.pubkey().as_ref()],
            &bubblegum_id,
        );

        // Calculate space and rent
        let space = bubblegum::get_merkle_tree_size(max_depth, max_buffer_size, canopy_depth);
        let rent = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(space)
            .await?;

        let create_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &tree_keypair.pubkey(),
            rent,
            space as u64,
            &bubblegum::spl_account_compression_id(),
        );

        let create_tree_ix = CreateTreeConfigBuilder::new()
            .tree_config(tree_config)
            .merkle_tree(tree_keypair.pubkey())
            .payer(self.payer.pubkey())
            .tree_creator(self.payer.pubkey())
            .log_wrapper(bubblegum::spl_noop_id())
            .compression_program(bubblegum::spl_account_compression_id())
            .system_program(solana_sdk::system_program::ID)
            .max_depth(max_depth)
            .max_buffer_size(max_buffer_size)
            .public(false)
            .instruction();

        let blockhash = self.rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[create_account_ix, create_tree_ix],
            Some(&self.payer.pubkey()),
            &[&*self.payer, &tree_keypair],
            blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&tx).await?;
        tracing::info!(
            "Created Merkle tree: {} (tx: {})",
            tree_keypair.pubkey(),
            signature
        );

        Ok((tree_keypair.pubkey(), signature.to_string()))
    }
}
