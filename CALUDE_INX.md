# Solana cNFT Minting Backend: Complete Architecture Specification

**A Rust/Axum server for agent-gated compressed NFT minting via Metaplex Bubblegum, inspired by the Claws NFT project's challenge-response pattern but using direct cNFT minting instead of Candy Machine.**

The system is a self-contained Rust backend that creates Merkle trees, generates math/logic challenges for AI agents, verifies solutions, and mints compressed NFTs directly through the Bubblegum program. Unlike Claws NFT (which uses Candy Machine v3 with a `thirdPartySigner` guard and Moltbook identity verification), this project controls the entire minting pipeline server-side — the server is the tree authority, the fee payer, and the sole signer. This eliminates the need for client counter-signing and delivers **cNFTs at ~$0.000005 per mint** compared to ~$0.024 for uncompressed Candy Machine mints.

---

## 1. System architecture overview

```
┌─────────────────────────────────────────────────────────┐
│                    AI Agent (Client)                     │
│  1. GET  /challenge       → receives math challenge     │
│  2. POST /mint            → submits answer + wallet     │
│  3. Receives tx signature → cNFT appears in wallet      │
└─────────────────┬───────────────────────────────────────┘
                  │ HTTPS
┌─────────────────▼───────────────────────────────────────┐
│              Axum HTTP Server (Rust)                     │
│                                                         │
│  ┌──────────┐  ┌──────────────┐  ┌────────────────┐    │
│  │ Routes   │  │ Challenge    │  │ Solana Service  │    │
│  │ /health  │  │ Service      │  │ - tree mgmt    │    │
│  │ /challenge│ │ - generate   │  │ - mint cNFTs   │    │
│  │ /mint    │  │ - verify     │  │ - send txns    │    │
│  │ /status  │  │ - expire     │  │ - PDA derive   │    │
│  └──────────┘  └──────────────┘  └────────────────┘    │
│                                                         │
│  ┌──────────────┐  ┌─────────────────────────────┐     │
│  │ Rate Limiter │  │ AppState (Arc<>)             │     │
│  │ (tower-      │  │ - RpcClient (async)          │     │
│  │  governor)   │  │ - Payer Keypair (Arc)         │     │
│  │              │  │ - SQLite Pool                 │     │
│  └──────────────┘  │ - Config (tree addr, etc.)    │     │
│                     └─────────────────────────────┘     │
│                                                         │
│  ┌──────────────────────────────────────────────┐      │
│  │ SQLite Database                               │      │
│  │ - challenges (nonce, expiry, status)          │      │
│  │ - mints (asset_id, tx_sig, recipient)         │      │
│  │ - merkle_trees (address, capacity, leaf_idx)  │      │
│  └──────────────────────────────────────────────┘      │
└─────────────────┬───────────────────────────────────────┘
                  │ RPC (JSON-RPC over HTTPS)
┌─────────────────▼───────────────────────────────────────┐
│              Solana Blockchain                           │
│                                                         │
│  ┌─────────────────┐  ┌───────────────────────────┐    │
│  │ Bubblegum       │  │ SPL Account Compression   │    │
│  │ Program         │  │ Program                   │    │
│  │ BGUMAp9Gq7i...  │  │ cmtDvXumGCr...            │    │
│  └─────────────────┘  └───────────────────────────┘    │
│                                                         │
│  ┌─────────────────┐  ┌───────────────────────────┐    │
│  │ Merkle Tree     │  │ Collection NFT            │    │
│  │ Account         │  │ (Token Metadata standard) │    │
│  └─────────────────┘  └───────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### How this differs from Claws NFT

| Aspect | Claws NFT (Candy Machine) | This Project (Bubblegum Direct) |
|--------|--------------------------|--------------------------------|
| **Minting program** | Candy Machine v3 + Candy Guard | Bubblegum `mintToCollectionV1` |
| **NFT type** | Uncompressed (Token Metadata) | Compressed NFT (cNFT) |
| **Cost per mint** | ~0.024 SOL | ~0.000005 SOL (tx fee only) |
| **Access control** | `thirdPartySigner` guard (on-chain) | Server-side challenge verification |
| **Agent verification** | Moltbook identity token | Math/logic challenge-response |
| **Transaction flow** | Server co-signs → client counter-signs → client submits | Server builds, signs, and submits entirely |
| **Supply enforcement** | Candy Machine item count (on-chain) | Server-side + tree capacity tracking |

---

## 2. API endpoints design

Three core endpoints plus health/status. The flow mirrors Claws' three-step pattern (read → challenge → mint) but simplified.

### `GET /health`

Health check. Returns server status, active tree address, and remaining capacity.

```json
// Response 200
{
  "status": "ok",
  "active_tree": "7nE4...xQk",
  "tree_capacity_remaining": 15892,
  "total_minted": 492,
  "version": "0.1.0"
}
```

### `GET /challenge`

Generates a math/logic challenge. Returns a challenge ID, the question, and expiration timestamp. Challenges expire after **5 minutes**.

```json
// Response 200
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "challenge_type": "math",
  "question": "What is 847 * 23 + 156?",
  "expires_at": "2026-02-08T15:05:00Z",
  "difficulty": "medium"
}
```

**Rate limit**: 10 challenges per minute per IP.

### `POST /mint`

Submits a challenge solution and wallet address. On success, the server mints a cNFT directly to the provided wallet and returns the transaction signature.

```json
// Request
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "answer": "19637",
  "wallet_address": "7nE4kBiH3X9Aq3pSfRbNqzPMjYp2QK1HxQk"
}

// Response 200
{
  "success": true,
  "tx_signature": "5UfD...3kQz",
  "asset_id": "BvR9...7mNp",
  "mint_index": 493,
  "message": "cNFT minted successfully"
}

// Response 400 (wrong answer)
{
  "success": false,
  "error": "incorrect_answer",
  "message": "Challenge answer is incorrect"
}

// Response 410 (expired)
{
  "success": false,
  "error": "challenge_expired",
  "message": "Challenge has expired. Request a new one."
}
```

**Rate limit**: 3 mint attempts per minute per IP.

### `GET /status/:tx_signature`

Checks the confirmation status of a mint transaction.

```json
// Response 200
{
  "tx_signature": "5UfD...3kQz",
  "status": "confirmed",
  "asset_id": "BvR9...7mNp",
  "recipient": "7nE4...xQk",
  "confirmed_at": "2026-02-08T15:01:12Z"
}
```

### `GET /skill.md`

Serves a markdown file describing the API for AI agents (following the Claws/OpenClaw skill pattern). This is a static file served from disk.

---

## 3. Rust project structure

Single crate layout (appropriate for this scope — a workspace split adds complexity without clear benefit for a single server binary):

```
cnft-mint-server/
├── Cargo.toml
├── .env.example
├── README.md
├── skill.md                          # Agent-facing API documentation
├── migrations/
│   └── 001_initial.sql               # SQLx migration
├── src/
│   ├── main.rs                       # Entry point: init config, DB, state, start Axum
│   ├── config.rs                     # Env var loading + Config struct
│   ├── state.rs                      # AppState definition
│   ├── error.rs                      # AppError enum + IntoResponse
│   ├── routes/
│   │   ├── mod.rs                    # Router construction
│   │   ├── health.rs                 # GET /health
│   │   ├── challenge.rs              # GET /challenge
│   │   ├── mint.rs                   # POST /mint
│   │   ├── status.rs                 # GET /status/:tx_signature
│   │   └── skill.rs                  # GET /skill.md (static file)
│   ├── services/
│   │   ├── mod.rs
│   │   ├── challenge.rs              # Challenge generation + verification
│   │   ├── solana.rs                 # RPC client, tx building, sending
│   │   ├── bubblegum.rs              # Bubblegum instruction builders
│   │   ├── tree_manager.rs           # Tree creation, rotation, capacity tracking
│   │   └── metadata.rs              # NFT metadata generation (name, URI, traits)
│   ├── models/
│   │   ├── mod.rs
│   │   ├── challenge.rs              # Challenge struct, ChallengeType enum
│   │   ├── mint.rs                   # MintRequest, MintResponse DTOs
│   │   └── tree.rs                   # TreeInfo struct
│   └── db/
│       ├── mod.rs
│       ├── challenges.rs             # Challenge CRUD operations
│       ├── mints.rs                  # Mint record CRUD
│       └── trees.rs                  # Tree record CRUD
└── tests/
    ├── challenge_tests.rs
    ├── mint_integration.rs
    └── helpers.rs
```

---

## 4. All required crates and dependencies

### Cargo.toml

```toml
[package]
name = "cnft-mint-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# ─── Solana / Metaplex ───
solana-sdk = "~2.1"
solana-client = "~2.1"
mpl-bubblegum = "2.1"
spl-account-compression = "1.0"
spl-noop = "1.0"
borsh = "0.10"
bs58 = "0.5"

# ─── Web Framework ───
axum = { version = "0.8", features = ["json", "macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "timeout", "fs"] }
tower-governor = "0.4"

# ─── Serialization ───
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"

# ─── Database ───
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "migrate", "chrono"] }

# ─── Crypto / Auth ───
rand = "0.8"
ed25519-dalek = "2"
base64 = "0.22"

# ─── Utilities ───
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
thiserror = "2"
anyhow = "1"

# ─── Logging ───
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Version compatibility notes

**All Solana crates must be pinned to the same `~2.1` line.** The `mpl-bubblegum 2.1.1` crate's dev dependencies use `solana-sdk ~2.1.21`, confirming compatibility. The `spl-account-compression 1.0.0` crate pulls in `anchor-lang 0.31` as a transitive dependency — this is expected and does not require direct use. **Do not use `solana-sdk 3.x`** as it introduces breaking changes incompatible with mpl-bubblegum 2.1.x. The `borsh` crate must stay in the `0.9–0.10` range since mpl-bubblegum requires `>=0.9, <1.0`.

---

## 5. Merkle tree setup and management strategy

### Program IDs (hardcode these as constants)

```rust
pub const BUBBLEGUM_PROGRAM_ID: &str = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY";
pub const SPL_ACCOUNT_COMPRESSION_ID: &str = "cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK";
pub const SPL_NOOP_ID: &str = "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV";
pub const TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
```

### Recommended tree configuration

| Scenario | maxDepth | maxBufferSize | canopyDepth | Capacity | Cost (SOL) |
|----------|----------|---------------|-------------|----------|------------|
| **Small (dev/test)** | 14 | 64 | 10 | 16,384 | ~0.68 |
| **Medium (production)** | 20 | 64 | 14 | 1,048,576 | ~7.7 |
| **Large (mass mint)** | 24 | 256 | 14 | 16,777,216 | ~7.67 |

**The constraint `maxDepth - canopyDepth ≤ 10` must be respected** for marketplace composability (Tensor, Magic Eden require proof length ≤ 10). For a starting collection, `maxDepth=14, maxBufferSize=64, canopyDepth=10` provides **16,384 cNFTs** at a one-time cost of ~0.68 SOL.

### Tree creation (two-step process)

Tree creation requires two instructions in a single transaction. The first allocates the Merkle tree account (owned by SPL Account Compression), the second initializes it via Bubblegum's `CreateTreeConfig` instruction which creates the Tree Config PDA.

```rust
use mpl_bubblegum::instructions::CreateTreeConfigBuilder;
use mpl_bubblegum::programs::{SPL_ACCOUNT_COMPRESSION_ID, SPL_NOOP_ID};
use solana_sdk::{
    pubkey::Pubkey, system_instruction, system_program,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction,
};
use solana_client::nonblocking::rpc_client::RpcClient;

pub async fn create_merkle_tree(
    rpc_client: &RpcClient,
    payer: &Keypair,
    max_depth: u32,
    max_buffer_size: u32,
    canopy_depth: u32,
) -> anyhow::Result<Pubkey> {
    let tree_keypair = Keypair::new();

    // Derive Tree Config PDA
    let (tree_config, _bump) = Pubkey::find_program_address(
        &[tree_keypair.pubkey().as_ref()],
        &mpl_bubblegum::ID,
    );

    // Step 1: Calculate space and create the account
    // Space = header + tree nodes + canopy nodes
    // Use spl_account_compression utilities or calculate manually
    let space = get_merkle_tree_size(max_depth, max_buffer_size, canopy_depth);
    let rent = rpc_client.get_minimum_balance_for_rent_exemption(space).await?;

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &tree_keypair.pubkey(),
        rent,
        space as u64,
        &spl_account_compression::ID,
    );

    // Step 2: Initialize via Bubblegum CreateTreeConfig
    let create_tree_ix = CreateTreeConfigBuilder::new()
        .tree_config(tree_config)
        .merkle_tree(tree_keypair.pubkey())
        .payer(payer.pubkey())
        .tree_creator(payer.pubkey())
        .log_wrapper(SPL_NOOP_ID)
        .compression_program(SPL_ACCOUNT_COMPRESSION_ID)
        .system_program(system_program::ID)
        .max_depth(max_depth)
        .max_buffer_size(max_buffer_size)
        .public(false) // Only server (tree creator) can mint
        .instruction();

    let blockhash = rpc_client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, create_tree_ix],
        Some(&payer.pubkey()),
        &[payer, &tree_keypair],
        blockhash,
    );

    rpc_client.send_and_confirm_transaction(&tx).await?;
    Ok(tree_keypair.pubkey())
}
```

### Tree rotation strategy

The server tracks `current_leaf_index` in the database for each tree. When `current_leaf_index >= 2^maxDepth`, the tree is full. The `tree_manager` service handles this:

1. Before each mint, check if the active tree has capacity
2. If full, create a new tree (same configuration) and mark the old tree as inactive
3. Update the database with the new active tree address
4. A single collection can span multiple trees — the collection NFT is independent of any specific tree

```rust
// Pseudocode for tree_manager.rs
pub async fn get_active_tree(&self) -> Result<TreeInfo> {
    let tree = self.db.get_active_tree().await?;
    match tree {
        Some(t) if t.current_leaf_index < t.max_capacity => Ok(t),
        _ => {
            // Create new tree, deactivate old one
            let new_tree_addr = self.create_merkle_tree(...).await?;
            self.db.insert_tree(new_tree_addr, ...).await?;
            if let Some(old) = tree {
                self.db.deactivate_tree(&old.address).await?;
            }
            self.db.get_active_tree().await?.ok_or(anyhow!("No active tree"))
        }
    }
}
```

---

## 6. cNFT minting flow (step by step)

### Complete flow from agent request to minted cNFT

```
Agent                          Server                          Solana
  │                              │                               │
  │  GET /challenge              │                               │
  │─────────────────────────────>│                               │
  │                              │ Generate math challenge       │
  │                              │ Store in DB (5-min expiry)    │
  │  { challenge_id, question }  │                               │
  │<─────────────────────────────│                               │
  │                              │                               │
  │  POST /mint                  │                               │
  │  { challenge_id, answer,     │                               │
  │    wallet_address }          │                               │
  │─────────────────────────────>│                               │
  │                              │ 1. Validate challenge exists  │
  │                              │ 2. Check not expired          │
  │                              │ 3. Verify answer              │
  │                              │ 4. Check wallet not duplicate │
  │                              │ 5. Get active tree            │
  │                              │ 6. Build metadata             │
  │                              │ 7. Build MintToCollectionV1   │
  │                              │    instruction                │
  │                              │                               │
  │                              │  send_and_confirm_transaction │
  │                              │──────────────────────────────>│
  │                              │                               │ Bubblegum
  │                              │                               │ appends leaf
  │                              │                               │ to Merkle tree
  │                              │         tx signature          │
  │                              │<──────────────────────────────│
  │                              │                               │
  │                              │ 8. Mark challenge consumed   │
  │                              │ 9. Record mint in DB          │
  │                              │ 10. Increment tree leaf index │
  │                              │                               │
  │  { success, tx_signature,    │                               │
  │    asset_id }                │                               │
  │<─────────────────────────────│                               │
```

### Building the MintToCollectionV1 instruction

This is the core minting instruction. The server is both the `payer` and the `tree_creator_or_delegate` (since `public: false`). The `collection_authority` must also sign — since the server created the collection NFT, the server's keypair serves as all three signers.

```rust
use mpl_bubblegum::instructions::MintToCollectionV1Builder;
use mpl_bubblegum::types::{
    Collection, Creator, MetadataArgs, TokenProgramVersion, TokenStandard,
};
use solana_sdk::pubkey::Pubkey;

pub fn build_mint_to_collection_ix(
    payer: &Pubkey,
    merkle_tree: &Pubkey,
    leaf_owner: &Pubkey,
    collection_mint: &Pubkey,
    name: String,
    uri: String,
    seller_fee_basis_points: u16,
) -> solana_sdk::instruction::Instruction {
    // Derive PDAs
    let (tree_config, _) = Pubkey::find_program_address(
        &[merkle_tree.as_ref()],
        &mpl_bubblegum::ID,
    );

    let (bubblegum_signer, _) = Pubkey::find_program_address(
        &[b"collection_cpi"],
        &mpl_bubblegum::ID,
    );

    // Collection metadata PDA (Token Metadata program)
    let (collection_metadata, _) = Pubkey::find_program_address(
        &[b"metadata", mpl_token_metadata::ID.as_ref(), collection_mint.as_ref()],
        &mpl_token_metadata::ID,
    );

    // Collection master edition PDA
    let (collection_edition, _) = Pubkey::find_program_address(
        &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            collection_mint.as_ref(),
            b"edition",
        ],
        &mpl_token_metadata::ID,
    );

    let metadata = MetadataArgs {
        name,
        symbol: "CNFT".to_string(),
        uri,
        seller_fee_basis_points,
        primary_sale_happened: false,
        is_mutable: true,
        edition_nonce: None,
        token_standard: Some(TokenStandard::NonFungible),
        collection: Some(Collection {
            verified: false, // Automatically set to true by the instruction
            key: *collection_mint,
        }),
        uses: None,
        token_program_version: TokenProgramVersion::Original,
        creators: vec![Creator {
            address: *payer,
            verified: true, // Server signs, so can be verified at mint
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
        .collection_authority_record_pda(mpl_bubblegum::ID) // Default when no delegated authority
        .collection_mint(*collection_mint)
        .collection_metadata(collection_metadata)
        .edition_account(collection_edition)
        .bubblegum_signer(bubblegum_signer)
        .log_wrapper(spl_noop::ID)
        .compression_program(spl_account_compression::ID)
        .token_metadata_program(mpl_token_metadata::ID)
        .system_program(system_program::ID)
        .metadata(metadata)
        .instruction()
}
```

### Full mint handler

```rust
pub async fn mint_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MintRequest>,
) -> Result<Json<MintResponse>, AppError> {
    // 1. Validate and consume challenge
    let challenge = state.db.get_challenge(&req.challenge_id).await?
        .ok_or(AppError::BadRequest("Challenge not found".into()))?;

    if challenge.status != "pending" {
        return Err(AppError::BadRequest("Challenge already used".into()));
    }
    if chrono::Utc::now() > challenge.expires_at {
        state.db.expire_challenge(&req.challenge_id).await?;
        return Err(AppError::Gone("Challenge expired".into()));
    }
    if !verify_challenge_answer(&challenge, &req.answer) {
        return Err(AppError::BadRequest("Incorrect answer".into()));
    }

    // 2. Parse wallet address
    let leaf_owner = Pubkey::from_str(&req.wallet_address)
        .map_err(|_| AppError::BadRequest("Invalid wallet address".into()))?;

    // 3. Get active tree (with rotation if needed)
    let tree_info = state.tree_manager.get_active_tree().await?;

    // 4. Generate metadata for this mint
    let mint_index = tree_info.current_leaf_index;
    let metadata_uri = format!("{}/{}.json", state.config.base_metadata_uri, mint_index);
    let name = format!("{} #{}", state.config.collection_name, mint_index);

    // 5. Build the instruction
    let mint_ix = build_mint_to_collection_ix(
        &state.payer.pubkey(),
        &tree_info.address,
        &leaf_owner,
        &state.config.collection_mint,
        name,
        metadata_uri,
        state.config.seller_fee_basis_points,
    );

    // 6. Build, sign, send transaction
    let blockhash = state.rpc_client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&state.payer.pubkey()),
        &[&*state.payer],
        blockhash,
    );
    let signature = state.rpc_client.send_and_confirm_transaction(&tx).await?;

    // 7. Derive asset ID
    let (asset_id, _) = Pubkey::find_program_address(
        &[b"asset", tree_info.address.as_ref(), &mint_index.to_le_bytes()],
        &mpl_bubblegum::ID,
    );

    // 8. Record in database
    state.db.mark_challenge_consumed(&req.challenge_id).await?;
    state.db.insert_mint(
        &asset_id.to_string(), &tree_info.address.to_string(),
        mint_index, &req.wallet_address, &metadata_uri,
        &signature.to_string(),
    ).await?;
    state.db.increment_tree_leaf_index(&tree_info.address.to_string()).await?;

    Ok(Json(MintResponse {
        success: true,
        tx_signature: signature.to_string(),
        asset_id: asset_id.to_string(),
        mint_index,
        message: "cNFT minted successfully".into(),
    }))
}
```

### Collection NFT setup (one-time, before tree creation)

Before minting cNFTs, create a standard Token Metadata collection NFT. This can be done via a CLI setup script or an admin endpoint. The collection NFT is a regular Metaplex NFT (not compressed) with `isCollection: true`. The server's payer keypair should be the update authority.

```rust
// This is a setup task — run once before the server starts minting.
// Can be implemented as a CLI subcommand: `cnft-mint-server setup-collection`
// Uses Token Metadata's CreateV1 instruction to create:
// 1. SPL Token mint account
// 2. Associated token account (mint 1 token)
// 3. Metadata PDA
// 4. Master Edition PDA
```

---

## 7. Challenge generation and verification logic

### Challenge types

The server generates math/logic challenges that AI agents can solve programmatically. Three difficulty tiers prevent trivial scripting while remaining solvable by legitimate agents.

```rust
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    Arithmetic,      // "What is 847 * 23 + 156?"
    ModularMath,     // "What is 2^17 mod 31?"
    LogicSequence,   // "What comes next: 2, 6, 18, 54, ?"
    WordMath,        // "If a=1, b=2... what is the sum of letters in 'SOLANA'?"
}

#[derive(Debug, Clone, Serialize)]
pub struct Challenge {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub question: String,
    pub answer: String,       // stored server-side, never sent to client
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: String,       // "pending", "consumed", "expired"
}

pub fn generate_challenge() -> Challenge {
    let mut rng = rand::thread_rng();
    let challenge_type = match rng.gen_range(0..4) {
        0 => ChallengeType::Arithmetic,
        1 => ChallengeType::ModularMath,
        2 => ChallengeType::LogicSequence,
        _ => ChallengeType::WordMath,
    };

    let (question, answer) = match challenge_type {
        ChallengeType::Arithmetic => {
            let a = rng.gen_range(100..999);
            let b = rng.gen_range(10..99);
            let c = rng.gen_range(10..999);
            let ops = ["+", "-", "*"];
            let op1 = ops[rng.gen_range(0..3)];
            let op2 = ops[rng.gen_range(0..3)];
            let result = eval_arithmetic(a, op1, b, op2, c);
            (
                format!("What is {} {} {} {} {}?", a, op1, b, op2, c),
                result.to_string(),
            )
        }
        ChallengeType::ModularMath => {
            let base = rng.gen_range(2..10);
            let exp = rng.gen_range(5..20);
            let modulus = rng.gen_range(7..50);
            let result = mod_pow(base, exp, modulus);
            (
                format!("What is {}^{} mod {}?", base, exp, modulus),
                result.to_string(),
            )
        }
        ChallengeType::LogicSequence => {
            // Geometric sequence: a, a*r, a*r^2, a*r^3, ?
            let a = rng.gen_range(1..10) as i64;
            let r = rng.gen_range(2..5) as i64;
            let seq: Vec<i64> = (0..4).map(|i| a * r.pow(i)).collect();
            let next = a * r.pow(4);
            (
                format!(
                    "What comes next in the sequence: {}, {}, {}, {}, ?",
                    seq[0], seq[1], seq[2], seq[3]
                ),
                next.to_string(),
            )
        }
        ChallengeType::WordMath => {
            let words = ["SOLANA", "MINT", "AGENT", "CHAIN", "TOKEN", "BLOCK"];
            let word = words[rng.gen_range(0..words.len())];
            let sum: u32 = word.chars()
                .map(|c| (c as u32) - ('A' as u32) + 1)
                .sum();
            (
                format!(
                    "If A=1, B=2, ..., Z=26, what is the sum of letter values in '{}'?",
                    word
                ),
                sum.to_string(),
            )
        }
    };

    Challenge {
        id: Uuid::new_v4().to_string(),
        challenge_type,
        question,
        answer,
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
        status: "pending".to_string(),
    }
}

pub fn verify_challenge_answer(challenge: &Challenge, submitted: &str) -> bool {
    challenge.answer.trim() == submitted.trim()
}
```

### Challenge expiration cleanup

Run a background task every 60 seconds to expire stale challenges:

```rust
// In main.rs, spawn a cleanup task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let _ = db_pool.execute(
            "UPDATE challenges SET status = 'expired'
             WHERE status = 'pending' AND expires_at < datetime('now')"
        ).await;
    }
});
```

---

## 8. Environment variables and configuration

### `.env.example`

```bash
# ─── Solana Configuration ───
SOLANA_RPC_URL=https://api.devnet.solana.com
# For mainnet, use a DAS-enabled RPC (Helius, Triton, QuickNode)
# SOLANA_RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY

# Server keypair (JSON byte array format, same as Solana CLI)
# This keypair is the tree creator, collection authority, and fee payer
PAYER_KEYPAIR=[174,47,154,16,202,...]

# ─── Tree Configuration ───
MERKLE_TREE_MAX_DEPTH=14
MERKLE_TREE_MAX_BUFFER_SIZE=64
MERKLE_TREE_CANOPY_DEPTH=10

# ─── Collection Configuration ───
COLLECTION_MINT=                    # Set after running setup-collection
COLLECTION_NAME=MyCNFTCollection
COLLECTION_SYMBOL=CNFT
BASE_METADATA_URI=https://arweave.net/your-metadata-folder
SELLER_FEE_BASIS_POINTS=500        # 5% royalty

# ─── Server Configuration ───
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite://data/cnft_mint.db

# ─── Rate Limiting ───
RATE_LIMIT_PER_SECOND=2
RATE_LIMIT_BURST=10
CHALLENGE_EXPIRY_SECONDS=300       # 5 minutes

# ─── Logging ───
RUST_LOG=cnft_mint_server=info,tower_http=debug
```

### Config struct

```rust
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
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
    pub challenge_expiry_seconds: i64,
}
```

---

## 9. Database schema

SQLite via `sqlx` with compile-time checked queries. Three core tables.

### `migrations/001_initial.sql`

```sql
-- Merkle tree tracking
CREATE TABLE IF NOT EXISTS merkle_trees (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    address     TEXT NOT NULL UNIQUE,
    max_depth   INTEGER NOT NULL,
    max_buffer_size INTEGER NOT NULL,
    canopy_depth    INTEGER NOT NULL,
    max_capacity    INTEGER NOT NULL,          -- 2^max_depth
    current_leaf_index INTEGER NOT NULL DEFAULT 0,
    collection_mint TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    creation_tx TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Challenge state machine: pending → consumed | expired
CREATE TABLE IF NOT EXISTS challenges (
    id              TEXT PRIMARY KEY,           -- UUID v4
    challenge_type  TEXT NOT NULL,
    question        TEXT NOT NULL,
    answer          TEXT NOT NULL,
    wallet_address  TEXT,                       -- optionally bind to wallet
    status          TEXT NOT NULL DEFAULT 'pending',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT NOT NULL,
    consumed_at     TEXT
);

-- Mint records (audit trail + asset tracking)
CREATE TABLE IF NOT EXISTS mints (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id        TEXT UNIQUE,
    tree_address    TEXT NOT NULL,
    leaf_index      INTEGER NOT NULL,
    recipient_wallet TEXT NOT NULL,
    metadata_uri    TEXT NOT NULL,
    metadata_name   TEXT,
    tx_signature    TEXT NOT NULL,
    challenge_id    TEXT,
    status          TEXT NOT NULL DEFAULT 'confirmed',  -- confirmed | failed
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (tree_address) REFERENCES merkle_trees(address),
    FOREIGN KEY (challenge_id) REFERENCES challenges(id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_challenges_status ON challenges(status, expires_at);
CREATE INDEX IF NOT EXISTS idx_mints_recipient ON mints(recipient_wallet);
CREATE INDEX IF NOT EXISTS idx_mints_tree ON mints(tree_address);
CREATE INDEX IF NOT EXISTS idx_trees_active ON merkle_trees(is_active);
```

---

## 10. Deployment considerations

### Server requirements

The binary is a single statically-linked Rust executable with an SQLite database file. **Minimum specs**: 1 vCPU, 512MB RAM, 1GB disk. The primary bottleneck is Solana RPC latency, not CPU or memory. A VPS (Hetzner, DigitalOcean, Railway) works well.

### Build and run

```bash
# Build release binary
cargo build --release

# Run database migrations
DATABASE_URL=sqlite://data/cnft_mint.db sqlx migrate run

# First-time setup: create collection NFT + initial Merkle tree
./target/release/cnft-mint-server setup

# Start server
./target/release/cnft-mint-server serve
```

### CLI subcommands (implement via `clap`)

```
cnft-mint-server setup              # Creates collection NFT + first Merkle tree
cnft-mint-server serve              # Starts the Axum HTTP server
cnft-mint-server create-tree        # Manually creates a new Merkle tree
cnft-mint-server status             # Shows current tree capacity, total mints
```

### Docker deployment

```dockerfile
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cnft-mint-server /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/skill.md /app/skill.md
WORKDIR /app
EXPOSE 3000
CMD ["cnft-mint-server", "serve"]
```

### RPC provider selection

**A DAS-enabled RPC is required** for reading cNFT data post-mint (verification, querying by owner). Standard Solana RPC nodes cannot query compressed NFT metadata. Recommended providers:

- **Helius** — best DAS documentation, free tier available, recommended for development and production
- **Triton** — co-developed the DAS spec, reliable
- **QuickNode** — DAS via marketplace add-on

For minting transactions, any standard RPC works. The DAS API is only needed for read operations (`getAsset`, `getAssetsByOwner`).

### Reverse proxy

Place behind nginx or Caddy for TLS termination:

```nginx
server {
    listen 443 ssl;
    server_name mint.yourproject.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## 11. Security model

### Keypair protection

The server's payer keypair is the **single most critical secret**. It controls the Merkle tree (tree creator authority), the collection (update authority), and pays transaction fees. Compromise means an attacker can mint unlimited cNFTs.

- **Load from environment variable** — never from a file on disk in production
- **Use a secrets manager** (AWS Secrets Manager, GCP Secret Manager, Vault) for production deployments
- **Principle of least privilege** — the keypair only needs enough SOL for transaction fees; keep the balance low and top up periodically
- **Consider separate keypairs** for fee payer vs tree authority (delegate the tree to a separate keypair via Bubblegum's `setTreeDelegate` so the primary keypair can be rotated)
- **Never log the keypair** — add tracing filters to exclude the `PAYER_KEYPAIR` env var

### Challenge-response security

| Threat | Mitigation |
|--------|-----------|
| **Replay attacks** | Each challenge ID is single-use; marked `consumed` after successful mint |
| **Brute force** | Rate limiting (3 mint attempts/min/IP); challenge answers are non-trivial |
| **Challenge hoarding** | 5-minute expiry; background cleanup task |
| **Bot spamming /challenge** | Rate limiting (10 challenges/min/IP); IP-based via tower-governor |
| **Answer enumeration** | Answers span large numeric ranges (arithmetic results 1–100K+) |

### Transaction security

- **Server signs and submits** — the client never sees a partially-signed transaction, eliminating transaction manipulation vectors
- **`skipPreflight: false`** — always simulate before sending to catch errors
- **Idempotency** — check if a challenge was already consumed before minting; store tx signature to prevent double-minting
- **Blockhash freshness** — fetch `getLatestBlockhash` immediately before signing; Solana transactions expire after ~60 seconds

### Network security

- **TLS everywhere** — serve behind nginx/Caddy with HTTPS
- **CORS** — restrict to known agent origins (or use permissive CORS if agents run from varied environments)
- **No admin endpoints exposed** — tree creation and setup are CLI-only, not HTTP-accessible
- **Helmet-style headers** — use tower-http to set security headers (`X-Content-Type-Options`, `X-Frame-Options`)

---

## 12. Appendix: Key PDA derivations reference

All PDAs used by the system, in one place for quick reference:

```rust
// Tree Config PDA (Bubblegum)
// Seeds: [merkle_tree_pubkey]
// Program: BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY
let (tree_config, _) = Pubkey::find_program_address(
    &[merkle_tree.as_ref()],
    &mpl_bubblegum::ID,
);

// Bubblegum Signer PDA (for MintToCollectionV1)
// Seeds: ["collection_cpi"]
// Program: BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY
let (bubblegum_signer, _) = Pubkey::find_program_address(
    &[b"collection_cpi"],
    &mpl_bubblegum::ID,
);

// Asset ID PDA (unique ID for each cNFT)
// Seeds: ["asset", merkle_tree_pubkey, leaf_index_le_bytes]
// Program: BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY
let (asset_id, _) = Pubkey::find_program_address(
    &[b"asset", merkle_tree.as_ref(), &leaf_index.to_le_bytes()],
    &mpl_bubblegum::ID,
);

// Collection Metadata PDA (Token Metadata)
// Seeds: ["metadata", token_metadata_program_id, collection_mint]
// Program: metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s
let (collection_metadata, _) = Pubkey::find_program_address(
    &[b"metadata", token_metadata_id.as_ref(), collection_mint.as_ref()],
    &token_metadata_id,
);

// Collection Master Edition PDA (Token Metadata)
// Seeds: ["metadata", token_metadata_program_id, collection_mint, "edition"]
// Program: metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s
let (collection_edition, _) = Pubkey::find_program_address(
    &[b"metadata", token_metadata_id.as_ref(), collection_mint.as_ref(), b"edition"],
    &token_metadata_id,
);
```

---

## 13. Appendix: Merkle tree account space calculation

The Merkle tree account size is not trivially computable — it depends on the internal layout of SPL Account Compression's `ConcurrentMerkleTree` struct. The recommended approach is to use the SPL library's size calculation or this formula:

```rust
/// Calculate the required account size for a concurrent Merkle tree.
/// Based on SPL Account Compression's getConcurrentMerkleTreeAccountSize.
fn get_merkle_tree_size(
    max_depth: u32,
    max_buffer_size: u32,
    canopy_depth: u32,
) -> usize {
    // Header: discriminator (8) + max_buffer_size (4) + max_depth (4)
    //       + authority (32) + creation_slot (8) + padding
    let header_size: usize = 8 + 32 + 32; // Approximate — check spl source

    // Changelog buffer: max_buffer_size entries × (max_depth + 1) × 32 bytes each
    let changelog_size = (max_buffer_size as usize) * ((max_depth as usize) + 1) * 32;

    // Right-most proof: max_depth × 32
    let rightmost_proof_size = (max_depth as usize) * 32;

    // Canopy: (2^(canopy_depth + 1) - 2) × 32 bytes
    let canopy_size = if canopy_depth > 0 {
        ((1usize << (canopy_depth + 1)) - 2) * 32
    } else {
        0
    };

    // Additional fields and alignment padding
    header_size + changelog_size + rightmost_proof_size + canopy_size + 256 // safety margin
}
```

**In practice, fetch the exact size at runtime** by querying `getMinimumBalanceForRentExemption` with a test allocation, or use the JavaScript `getConcurrentMerkleTreeAccountSize` function from `@solana/spl-account-compression` during setup and hardcode the result. For `maxDepth=14, maxBufferSize=64, canopyDepth=10`, the account size is approximately **97,272 bytes** costing **~0.68 SOL** in rent.

---

## 14. Appendix: Complete AppState and main.rs skeleton

```rust
// src/state.rs
use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::keypair::Keypair;
use sqlx::SqlitePool;
use crate::config::Config;
use crate::services::tree_manager::TreeManager;

pub struct AppState {
    pub config: Config,
    pub rpc_client: RpcClient,
    pub payer: Arc<Keypair>,
    pub db: SqlitePool,
    pub tree_manager: TreeManager,
}
```

```rust
// src/main.rs
use std::sync::Arc;
use std::net::SocketAddr;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_governor::{GovernorConfigBuilder, GovernorLayer};

mod config;
mod state;
mod error;
mod routes;
mod services;
mod models;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env()?;

    // Initialize database
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    // Load server keypair
    let key_bytes: Vec<u8> = serde_json::from_str(&config.payer_keypair_json)?;
    let payer = Arc::new(Keypair::from_bytes(&key_bytes)?);
    tracing::info!("Server pubkey: {}", payer.pubkey());

    // Initialize Solana RPC client (async/nonblocking)
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(
        config.solana_rpc_url.clone()
    );

    // Initialize tree manager
    let tree_manager = services::tree_manager::TreeManager::new(
        db.clone(), rpc_client.clone(), payer.clone(), config.clone()
    );

    let state = Arc::new(state::AppState {
        config: config.clone(),
        rpc_client,
        payer,
        db: db.clone(),
        tree_manager,
    });

    // Rate limiter
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(config.rate_limit_per_second)
        .burst_size(config.rate_limit_burst)
        .finish()
        .unwrap();

    // Build router
    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(GovernorLayer::new(&governor_conf));

    // Spawn challenge cleanup task
    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let _ = sqlx::query(
                "UPDATE challenges SET status = 'expired'
                 WHERE status = 'pending' AND expires_at < datetime('now')"
            )
            .execute(&cleanup_db)
            .await;
        }
    });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await?;

    Ok(())
}
```

```rust
// src/routes/mod.rs
use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use crate::state::AppState;

pub mod health;
pub mod challenge;
pub mod mint;
pub mod status;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/challenge", get(challenge::challenge_handler))
        .route("/mint", post(mint::mint_handler))
        .route("/status/{tx_signature}", get(status::status_handler))
        .route("/skill.md", get(|| async {
            tokio::fs::read_to_string("skill.md").await.unwrap_or_default()
        }))
        .with_state(state)
}
```

---

This specification covers every component needed to build the system end-to-end. The architecture prioritizes simplicity (single binary, SQLite, server-controlled minting) while remaining production-capable. The critical design decision — using `public: false` trees with server-side minting rather than Candy Machine guards — gives full control over the minting pipeline at dramatically lower cost per NFT. Feed this document to Claude Code with the instruction to implement each module following the code patterns and structures described above.
