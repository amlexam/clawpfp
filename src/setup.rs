use std::sync::Arc;
use borsh::BorshSerialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    system_instruction,
    system_program,
    transaction::Transaction,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::str::FromStr;

// Program IDs
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const RENT_SYSVAR_ID: &str = "SysvarRent111111111111111111111111111111111";

const SPL_TOKEN_MINT_SIZE: u64 = 82;

fn token_program_id() -> Pubkey {
    Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap()
}

fn associated_token_program_id() -> Pubkey {
    Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap()
}

fn token_metadata_program_id() -> Pubkey {
    Pubkey::from_str(TOKEN_METADATA_PROGRAM_ID).unwrap()
}

fn rent_sysvar_id() -> Pubkey {
    Pubkey::from_str(RENT_SYSVAR_ID).unwrap()
}

// ─── SPL Token instruction builders ───

fn initialize_mint2_ix(
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Instruction {
    // InitializeMint2 = discriminator 20
    let mut data = vec![20u8, decimals];
    data.extend_from_slice(mint_authority.as_ref());
    match freeze_authority {
        Some(fa) => {
            data.push(1); // COption::Some
            data.extend_from_slice(fa.as_ref());
        }
        None => {
            data.push(0); // COption::None
            data.extend_from_slice(&[0u8; 32]);
        }
    }

    Instruction {
        program_id: token_program_id(),
        accounts: vec![
            AccountMeta::new(*mint, false),
        ],
        data,
    }
}

fn mint_to_ix(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    // MintTo = discriminator 7
    let mut data = vec![7u8];
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: token_program_id(),
        accounts: vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(*authority, true),
        ],
        data,
    }
}

// ─── Associated Token Account ───

fn derive_ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    let (ata, _) = Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            token_program_id().as_ref(),
            mint.as_ref(),
        ],
        &associated_token_program_id(),
    );
    ata
}

fn create_ata_idempotent_ix(
    funder: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    let ata = derive_ata(wallet, mint);
    // CreateIdempotent = discriminator 1
    Instruction {
        program_id: associated_token_program_id(),
        accounts: vec![
            AccountMeta::new(*funder, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*wallet, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token_program_id(), false),
        ],
        data: vec![1],
    }
}

// ─── Token Metadata instruction data (borsh) ───

#[derive(BorshSerialize)]
struct CreateMetadataAccountArgsV3 {
    data: DataV2,
    is_mutable: bool,
    collection_details: Option<CollectionDetails>,
}

#[derive(BorshSerialize)]
struct DataV2 {
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    creators: Option<Vec<CreatorData>>,
    collection: Option<CollectionData>,
    uses: Option<UsesData>,
}

#[derive(BorshSerialize)]
struct CreatorData {
    address: [u8; 32],
    verified: bool,
    share: u8,
}

#[allow(dead_code)]
#[derive(BorshSerialize)]
struct CollectionData {
    verified: bool,
    key: [u8; 32],
}

#[allow(dead_code)]
#[derive(BorshSerialize)]
struct UsesData {
    use_method: u8,
    remaining: u64,
    total: u64,
}

#[derive(BorshSerialize)]
enum CollectionDetails {
    V1 { size: u64 },
}

#[derive(BorshSerialize)]
struct CreateMasterEditionArgs {
    max_supply: Option<u64>,
}

// ─── Token Metadata instruction builders ───

fn derive_metadata_pda(mint: &Pubkey) -> Pubkey {
    let tm_id = token_metadata_program_id();
    let (pda, _) = Pubkey::find_program_address(
        &[b"metadata", tm_id.as_ref(), mint.as_ref()],
        &tm_id,
    );
    pda
}

fn derive_master_edition_pda(mint: &Pubkey) -> Pubkey {
    let tm_id = token_metadata_program_id();
    let (pda, _) = Pubkey::find_program_address(
        &[b"metadata", tm_id.as_ref(), mint.as_ref(), b"edition"],
        &tm_id,
    );
    pda
}

fn create_metadata_account_v3_ix(
    metadata: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    args: CreateMetadataAccountArgsV3,
) -> Instruction {
    // Instruction discriminator for CreateMetadataAccountV3 = 33
    let mut data = vec![33u8];
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id: token_metadata_program_id(),
        accounts: vec![
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(rent_sysvar_id(), false),
        ],
        data,
    }
}

fn create_master_edition_v3_ix(
    edition: &Pubkey,
    mint: &Pubkey,
    update_authority: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    metadata: &Pubkey,
) -> Instruction {
    // Instruction discriminator for CreateMasterEditionV3 = 17
    let mut data = vec![17u8];
    let args = CreateMasterEditionArgs {
        max_supply: Some(0),
    };
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id: token_metadata_program_id(),
        accounts: vec![
            AccountMeta::new(*edition, false),
            AccountMeta::new(*mint, false),
            AccountMeta::new_readonly(*update_authority, true),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(token_program_id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(rent_sysvar_id(), false),
        ],
        data,
    }
}

// ─── Main setup function ───

pub async fn setup_collection(
    rpc_client: &RpcClient,
    payer: &Keypair,
    name: &str,
    symbol: &str,
    uri: &str,
) -> anyhow::Result<Pubkey> {
    let collection_mint = Keypair::new();
    let mint_pubkey = collection_mint.pubkey();
    let payer_pubkey = payer.pubkey();

    tracing::info!("Creating collection mint: {}", mint_pubkey);

    // Derive addresses
    let ata = derive_ata(&payer_pubkey, &mint_pubkey);
    let metadata_pda = derive_metadata_pda(&mint_pubkey);
    let master_edition_pda = derive_master_edition_pda(&mint_pubkey);

    // Calculate rent for mint account
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(SPL_TOKEN_MINT_SIZE as usize)
        .await?;

    // Transaction 1: Create mint, ATA, mint 1 token
    let ix1 = system_instruction::create_account(
        &payer_pubkey,
        &mint_pubkey,
        rent,
        SPL_TOKEN_MINT_SIZE,
        &token_program_id(),
    );
    let ix2 = initialize_mint2_ix(&mint_pubkey, &payer_pubkey, Some(&payer_pubkey), 0);
    let ix3 = create_ata_idempotent_ix(&payer_pubkey, &payer_pubkey, &mint_pubkey);
    let ix4 = mint_to_ix(&mint_pubkey, &ata, &payer_pubkey, 1);

    // Transaction 1: Create metadata
    let ix5 = create_metadata_account_v3_ix(
        &metadata_pda,
        &mint_pubkey,
        &payer_pubkey,
        &payer_pubkey,
        &payer_pubkey,
        CreateMetadataAccountArgsV3 {
            data: DataV2 {
                name: name.to_string(),
                symbol: symbol.to_string(),
                uri: uri.to_string(),
                seller_fee_basis_points: 500,
                creators: Some(vec![CreatorData {
                    address: payer_pubkey.to_bytes(),
                    verified: true,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: Some(CollectionDetails::V1 { size: 0 }),
        },
    );

    // Create master edition
    let ix6 = create_master_edition_v3_ix(
        &master_edition_pda,
        &mint_pubkey,
        &payer_pubkey,
        &payer_pubkey,
        &payer_pubkey,
        &metadata_pda,
    );

    let blockhash = rpc_client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[ix1, ix2, ix3, ix4, ix5, ix6],
        Some(&payer_pubkey),
        &[payer, &collection_mint],
        blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&tx).await?;
    tracing::info!("Collection created! Mint: {}, Tx: {}", mint_pubkey, signature);

    Ok(mint_pubkey)
}
