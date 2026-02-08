use mpl_bubblegum::instructions::MintToCollectionV1Builder;
use mpl_bubblegum::types::{
    Collection, Creator, MetadataArgs, TokenProgramVersion, TokenStandard,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use std::str::FromStr;

pub const BUBBLEGUM_PROGRAM_ID: &str = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY";
pub const SPL_ACCOUNT_COMPRESSION_ID: &str = "cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK";
pub const SPL_NOOP_ID: &str = "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV";
pub const TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

pub fn bubblegum_program_id() -> Pubkey {
    Pubkey::from_str(BUBBLEGUM_PROGRAM_ID).unwrap()
}

pub fn spl_account_compression_id() -> Pubkey {
    Pubkey::from_str(SPL_ACCOUNT_COMPRESSION_ID).unwrap()
}

pub fn spl_noop_id() -> Pubkey {
    Pubkey::from_str(SPL_NOOP_ID).unwrap()
}

pub fn token_metadata_program_id() -> Pubkey {
    Pubkey::from_str(TOKEN_METADATA_PROGRAM_ID).unwrap()
}

pub fn build_mint_to_collection_ix(
    payer: &Pubkey,
    merkle_tree: &Pubkey,
    leaf_owner: &Pubkey,
    collection_mint: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
) -> solana_sdk::instruction::Instruction {
    let bubblegum_id = bubblegum_program_id();
    let token_metadata_id = token_metadata_program_id();

    // Derive PDAs
    let (tree_config, _) = Pubkey::find_program_address(
        &[merkle_tree.as_ref()],
        &bubblegum_id,
    );

    let (bubblegum_signer, _) = Pubkey::find_program_address(
        &[b"collection_cpi"],
        &bubblegum_id,
    );

    let (collection_metadata, _) = Pubkey::find_program_address(
        &[b"metadata", token_metadata_id.as_ref(), collection_mint.as_ref()],
        &token_metadata_id,
    );

    let (collection_edition, _) = Pubkey::find_program_address(
        &[
            b"metadata",
            token_metadata_id.as_ref(),
            collection_mint.as_ref(),
            b"edition",
        ],
        &token_metadata_id,
    );

    let metadata = MetadataArgs {
        name,
        symbol,
        uri,
        seller_fee_basis_points,
        primary_sale_happened: false,
        is_mutable: true,
        edition_nonce: None,
        token_standard: Some(TokenStandard::NonFungible),
        collection: Some(Collection {
            verified: false,
            key: *collection_mint,
        }),
        uses: None,
        token_program_version: TokenProgramVersion::Original,
        creators: vec![Creator {
            address: *payer,
            verified: true,
            share: 100,
        }],
    };

    MintToCollectionV1Builder::new()
        .tree_config(tree_config)
        .leaf_owner(*leaf_owner)
        .leaf_delegate(*leaf_owner)
        .merkle_tree(*merkle_tree)
        .payer(*payer)
        .tree_creator_or_delegate(*payer)
        .collection_authority(*payer)
        .collection_authority_record_pda(Some(bubblegum_id))
        .collection_mint(*collection_mint)
        .collection_metadata(collection_metadata)
        .collection_edition(collection_edition)
        .bubblegum_signer(bubblegum_signer)
        .log_wrapper(spl_noop_id())
        .compression_program(spl_account_compression_id())
        .token_metadata_program(token_metadata_id)
        .system_program(system_program::ID)
        .metadata(metadata)
        .instruction()
}

/// Derive the asset ID for a cNFT given the tree address and leaf index
pub fn derive_asset_id(merkle_tree: &Pubkey, leaf_index: u64) -> Pubkey {
    let (asset_id, _) = Pubkey::find_program_address(
        &[b"asset", merkle_tree.as_ref(), &leaf_index.to_le_bytes()],
        &bubblegum_program_id(),
    );
    asset_id
}

/// Calculate the required account size for a concurrent Merkle tree.
/// Matches the exact layout of SPL Account Compression's ConcurrentMerkleTree.
pub fn get_merkle_tree_size(
    max_depth: u32,
    max_buffer_size: u32,
    canopy_depth: u32,
) -> usize {
    // Header: account_type(1) + version(1) + ConcurrentMerkleTreeHeaderDataV1(54) = 56
    let header_size: usize = 56;

    // ConcurrentMerkleTree struct:
    //   sequence_number(8) + active_index(8) + buffer_size(8) = 24
    let tree_meta: usize = 24;

    // ChangeLog entry: root(32) + path_nodes(MAX_DEPTH*32) + index(4) + padding(4)
    let changelog_entry_size = 40 + (max_depth as usize) * 32;
    let changelog_size = (max_buffer_size as usize) * changelog_entry_size;

    // Rightmost proof: proof(MAX_DEPTH*32) + leaf(32) + index(4) + padding(4)
    let rightmost_proof_size = 40 + (max_depth as usize) * 32;

    // Canopy: (2^(canopy_depth + 1) - 2) * 32
    let canopy_size = if canopy_depth > 0 {
        ((1usize << (canopy_depth + 1)) - 2) * 32
    } else {
        0
    };

    header_size + tree_meta + changelog_size + rightmost_proof_size + canopy_size
}
