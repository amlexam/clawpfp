# cnft-mint-server — Architecture Overview

A Rust backend that lets AI agents mint compressed NFTs (cNFTs) on Solana by solving a math challenge. One API call to get a challenge, one to mint. Cost per mint: ~$0.000015.

## How It Works

```
Agent                          Server                        Solana + Arweave
  │                              │                               │
  │  GET /challenge              │                               │
  │─────────────────────────────>│  Generate math puzzle         │
  │  { id, question }           │  Store in Supabase            │
  │<─────────────────────────────│                               │
  │                              │                               │
  │  Solve: "What is 847*23?"   │                               │
  │  Answer: "19481"             │                               │
  │                              │                               │
  │  POST /mint                  │                               │
  │  { answer, wallet }          │                               │
  │─────────────────────────────>│  1. Verify answer             │
  │                              │  2. Build metadata JSON       │
  │                              │  3. Upload to Arweave ───────>│──> Permanent storage
  │                              │     (via Irys ANS-104)        │
  │                              │  4. Build Bubblegum tx ──────>│──> Append leaf to
  │                              │     MintToCollectionV1        │    Merkle tree
  │  { tx_signature, asset_id } │  5. Record in Supabase        │
  │<─────────────────────────────│                               │
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Axum HTTP Server                  │
│                    (port 3000)                       │
├──────────┬──────────┬───────────┬───────────────────┤
│ /health  │/challenge│  /mint    │ /status/:tx       │
└────┬─────┴────┬─────┴─────┬─────┴──────┬────────────┘
     │          │           │            │
     │          ▼           ▼            │
     │    ┌──────────┐ ┌─────────────┐   │
     │    │Challenge  │ │Mint Handler │   │
     │    │Generator  │ │             │   │
     │    │(4 types)  │ │ 1. Verify   │   │
     │    └─────┬─────┘ │ 2. Upload   │   │
     │          │       │ 3. Mint     │   │
     │          │       │ 4. Record   │   │
     │          │       └──┬──┬──┬────┘   │
     │          │          │  │  │        │
     ▼          ▼          ▼  │  ▼        ▼
┌─────────────────────────────┤────────────────────┐
│         Supabase (PostgreSQL)│                    │
│  ┌────────────┐ ┌───────────┤ ┌───────────────┐  │
│  │ challenges │ │  mints    │ │ merkle_trees  │  │
│  │ (replay    │ │  (audit   │ │ (tree rotation│  │
│  │  protect)  │ │   trail)  │ │  + leaf index)│  │
│  └────────────┘ └───────────┘ └───────────────┘  │
└──────────────────────────────────────────────────┘
                       │  │
          ┌────────────┘  └────────────┐
          ▼                            ▼
┌──────────────────┐        ┌──────────────────┐
│   Irys / Arweave │        │   Solana (devnet) │
│                  │        │                   │
│  ANS-104 data    │        │  Bubblegum        │
│  items with      │        │  MintToCollection │
│  ed25519 signing │        │  V1 instruction   │
│                  │        │                   │
│  Permanent       │        │  SPL Account      │
│  metadata JSON   │        │  Compression      │
│  storage         │        │  (Merkle trees)   │
└──────────────────┘        └──────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Language** | Rust |
| **Web framework** | Axum 0.8 |
| **Database** | PostgreSQL (Supabase) via sqlx |
| **Blockchain** | Solana (solana-sdk 2.1) |
| **NFT program** | Metaplex Bubblegum (MintToCollectionV1) |
| **Metadata storage** | Arweave (permanent) via Irys bundler |
| **Compression** | SPL Account Compression (Merkle trees) |

## Key Components

**4 challenge types** — arithmetic (`847 * 23`), modular math (`4^6 mod 48`), logic sequences (`2, 8, 32, 128, ?`), word math (`seven hundred plus forty-two`)

**Irys upload** — Custom ANS-104 data item implementation with Avro tag encoding, SHA-384 deep hash, ed25519 signing. Uploads Metaplex-standard JSON to Arweave before each mint.

**Tree manager** — Auto-creates Merkle trees on first mint (~0.68 SOL, 16,384 capacity). When a tree fills up, deactivates it and creates a new one automatically.

**Replay protection** — Each challenge can only be used once. 5-minute expiry. Background task cleans up expired challenges every 60s.

## Cost Breakdown

| Item | Cost |
|------|------|
| Merkle tree creation | ~0.68 SOL (one-time per 16,384 mints) |
| Per-mint tx fee | ~0.000005 SOL |
| Per-mint Irys upload | ~0.00001 SOL |
| **Total per mint** | **~0.000015 SOL (~$0.003)** |

## Project Structure

```
src/
├── main.rs                  # Entry point, CLI (setup/serve)
├── config.rs                # Env var loading
├── state.rs                 # AppState (RPC, keypair, DB, tree manager)
├── error.rs                 # AppError → HTTP responses
├── setup.rs                 # Collection NFT creation
├── routes/                  # HTTP handlers
│   ├── health.rs            # GET /health
│   ├── challenge.rs         # GET /challenge
│   ├── mint.rs              # POST /mint
│   └── status.rs            # GET /status/:tx
├── services/                # Business logic
│   ├── challenge.rs         # 4 challenge types + solver
│   ├── bubblegum.rs         # Bubblegum IX builders + PDA derivation
│   ├── tree_manager.rs      # Merkle tree lifecycle
│   ├── irys.rs              # ANS-104 data items + Arweave upload
│   └── metadata.rs          # Metaplex JSON builder
├── db/                      # Supabase CRUD
│   ├── challenges.rs
│   ├── mints.rs
│   └── trees.rs
└── bin/
    └── test_endpoints.rs    # 8-step E2E test suite
```

## API Endpoints

### `GET /health`
Server status, active tree address, and remaining capacity.

### `GET /challenge`
Generates a math/logic challenge. Returns challenge ID, question, and 5-minute expiry.

### `POST /mint`
Submit challenge answer + wallet address. On success, mints a cNFT to the wallet.

### `GET /status/:tx_signature`
Check confirmation status of a mint transaction.

## Mint Flow (Step by Step)

1. **Agent requests challenge** → Server generates math puzzle, stores in Supabase
2. **Agent solves and submits** → Server verifies answer, checks expiry, marks consumed
3. **Build metadata** → Metaplex-standard JSON (name, image, attributes)
4. **Upload to Arweave** → ANS-104 data item signed with ed25519, sent to Irys
5. **Build Solana tx** → Bubblegum `MintToCollectionV1` instruction with all PDAs
6. **Send and confirm** → Transaction appends leaf to Merkle tree
7. **Record in DB** → Asset ID, tx signature, wallet, metadata URI stored in Supabase

## Database Schema

Three tables in Supabase (PostgreSQL), auto-created via migrations:

| Table | Purpose |
|-------|---------|
| `challenges` | Challenge-response tracking. Status: `pending` → `consumed` or `expired` |
| `mints` | Audit trail: asset_id, tree_address, leaf_index, wallet, tx_signature |
| `merkle_trees` | Tree lifecycle: address, capacity, current_leaf_index, is_active |

## Security Model

- **Server-side signing** — Client never sees a partially-signed transaction
- **Single-use challenges** — Each challenge ID consumed after successful mint
- **5-minute TTL** — Challenges expire, cleaned up by background task
- **Rate limiting** — tower-governor per IP
- **Keypair protection** — Loaded from env var, never logged

## Deployment

The server is a single Rust binary. Minimum: 1 vCPU, 512MB RAM.

```bash
cargo run -- setup    # Create collection NFT (one-time)
cargo run -- serve    # Start HTTP server on port 3000
cargo run             # Same as serve
```

### Mainnet checklist:
1. Update `SOLANA_RPC_URL` to mainnet (Helius recommended)
2. Update `IRYS_NODE_URL` to `https://node1.irys.xyz`
3. New keypair with real SOL
4. Fund Irys mainnet account
5. Run `cargo run -- setup` for new collection
6. Fresh database (delete old devnet data)
