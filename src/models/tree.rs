use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct TreeInfo {
    pub address: Pubkey,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
    pub max_capacity: u64,
    pub current_leaf_index: u64,
    pub is_active: bool,
}

/// Database row representation for merkle_trees table
#[derive(Debug, Clone)]
pub struct TreeRow {
    pub id: i64,
    pub address: String,
    pub max_depth: i64,
    pub max_buffer_size: i64,
    pub canopy_depth: i64,
    pub max_capacity: i64,
    pub current_leaf_index: i64,
    pub is_active: bool,
}

impl TryFrom<TreeRow> for TreeInfo {
    type Error = anyhow::Error;

    fn try_from(row: TreeRow) -> Result<Self, Self::Error> {
        Ok(TreeInfo {
            address: Pubkey::from_str(&row.address)?,
            max_depth: row.max_depth as u32,
            max_buffer_size: row.max_buffer_size as u32,
            canopy_depth: row.canopy_depth as u32,
            max_capacity: row.max_capacity as u64,
            current_leaf_index: row.current_leaf_index as u64,
            is_active: row.is_active,
        })
    }
}
