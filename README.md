# cnft-mint-server

A Rust/Axum backend for agent-gated compressed NFT (cNFT) minting on Solana via Metaplex Bubblegum. AI agents solve a math challenge to prove liveness, then the server mints a cNFT directly to their wallet at ~$0.000005 per mint.

## Prerequisites

- **Rust** (1.75+) — [install](https://rustup.rs)
- **Solana keypair** — JSON byte array format (same as `solana-keygen new`)
- **Funded wallet** — ~1 SOL on devnet for tree creation + tx fees
- **RPC endpoint** — any Solana RPC; [Helius](https://helius.dev) recommended for DAS support

## Quick start

```bash
# 1. Clone and build
git clone <repo-url> && cd cnft-mint-server
cargo build

# 2. Configure
cp .env.example .env
# Edit .env — set PAYER_KEYPAIR and SOLANA_RPC_URL at minimum

# 3. Create collection NFT (one-time setup)
cargo run -- setup

# 4. Copy the printed COLLECTION_MINT into your .env

# 5. Start the server
cargo run -- serve
```

## Configuration

All config is via environment variables (`.env` file loaded automatically):

| Variable | Default | Description |
|----------|---------|-------------|
| `SOLANA_RPC_URL` | `https://api.devnet.solana.com` | Solana RPC endpoint |
| `PAYER_KEYPAIR` | — | Server keypair as JSON byte array `[12,34,...]` |
| `COLLECTION_MINT` | — | Collection NFT address (from `cargo run -- setup`) |
| `COLLECTION_NAME` | `MyCNFTCollection` | Name prefix for minted cNFTs |
| `COLLECTION_SYMBOL` | `CNFT` | Token symbol |
| `BASE_METADATA_URI` | — | Base URI for off-chain JSON metadata |
| `SELLER_FEE_BASIS_POINTS` | `500` | Royalty (500 = 5%) |
| `MERKLE_TREE_MAX_DEPTH` | `14` | Tree depth (14 = 16,384 cNFTs) |
| `MERKLE_TREE_MAX_BUFFER_SIZE` | `64` | Concurrent update buffer |
| `MERKLE_TREE_CANOPY_DEPTH` | `10` | On-chain proof cache depth |
| `PORT` | `3000` | HTTP port |
| `DATABASE_URL` | `sqlite://data/cnft_mint.db` | SQLite database path |
| `CHALLENGE_EXPIRY_SECONDS` | `300` | Challenge TTL (5 min) |

## API endpoints

### `GET /health`

Server status, active tree address, and remaining capacity.

```bash
curl http://localhost:3000/health
```

```json
{
  "status": "ok",
  "active_tree": "CvNByAg...",
  "tree_capacity_remaining": 16383,
  "total_minted": 1,
  "version": "0.1.0"
}
```

### `GET /challenge`

Get a math/logic challenge. Expires after 5 minutes.

```bash
curl http://localhost:3000/challenge
```

```json
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "challenge_type": "arithmetic",
  "question": "What is 847 * 23 + 156?",
  "expires_at": "2026-02-08T15:05:00Z",
  "difficulty": "medium"
}
```

Challenge types: `arithmetic`, `modular_math`, `logic_sequence`, `word_math`.

### `POST /mint`

Submit challenge answer + wallet address. On success, a cNFT is minted to the wallet.

```bash
curl -X POST http://localhost:3000/mint \
  -H "Content-Type: application/json" \
  -d '{
    "challenge_id": "550e8400-...",
    "answer": "19637",
    "wallet_address": "7nE4kBiH3X..."
  }'
```

```json
{
  "success": true,
  "tx_signature": "5UfD...3kQz",
  "asset_id": "BvR9...7mNp",
  "mint_index": 1,
  "message": "cNFT minted successfully"
}
```

The first mint auto-creates a Merkle tree (~0.68 SOL, ~30s). Subsequent mints are near-instant.

### `GET /status/:tx_signature`

Check confirmation status of a mint transaction.

```bash
curl http://localhost:3000/status/5UfD...3kQz
```

## Testing

Run the built-in end-to-end test suite against a running server:

```bash
# Terminal 1 — start the server
cargo run -- serve

# Terminal 2 — run tests
cargo run --bin test_endpoints
```

The test binary exercises the full flow automatically:

1. Health check
2. Get challenge
3. Solve challenge
4. Submit wrong answer (expects 400)
5. Submit correct answer (mints cNFT on-chain)
6. Verify transaction status
7. Replay protection (expects 400)
8. Health check post-mint

```
  RESULTS: 8 passed, 0 failed  (28.5s total)
```

## CLI commands

```
cargo run -- setup     # Create collection NFT + prints COLLECTION_MINT
cargo run -- serve     # Start the HTTP server
cargo run              # Same as serve
```

## Architecture

```
Agent                          Server                          Solana
  │  GET /challenge              │                               │
  │─────────────────────────────>│  Generate math challenge      │
  │  { challenge_id, question }  │  Store in SQLite              │
  │<─────────────────────────────│                               │
  │                              │                               │
  │  POST /mint { answer, wallet}│                               │
  │─────────────────────────────>│  Verify answer                │
  │                              │  Build MintToCollectionV1     │
  │                              │──────────────────────────────>│
  │                              │         tx signature          │  Bubblegum appends
  │                              │<──────────────────────────────│  leaf to Merkle tree
  │  { tx_signature, asset_id }  │                               │
  │<─────────────────────────────│                               │
```

- **Minting program**: Bubblegum `MintToCollectionV1` (not Candy Machine)
- **Cost per mint**: ~0.000005 SOL (tx fee only)
- **Tree rotation**: automatic when a tree fills up
- **Database**: SQLite (challenges, mints, tree tracking)
- **Challenge expiry**: background task cleans up every 60s

## Project structure

```
src/
├── main.rs                  # Entry point, CLI routing, server startup
├── config.rs                # Env var loading
├── state.rs                 # AppState (RPC, keypair, DB, tree manager)
├── error.rs                 # AppError → HTTP responses
├── setup.rs                 # Collection NFT creation
├── routes/                  # Axum handlers
│   ├── health.rs
│   ├── challenge.rs
│   ├── mint.rs
│   └── status.rs
├── services/                # Business logic
│   ├── challenge.rs         # Challenge generation & verification
│   ├── bubblegum.rs         # Bubblegum instruction builders
│   ├── tree_manager.rs      # Merkle tree lifecycle
│   ├── solana.rs            # RPC helpers
│   └── metadata.rs          # NFT name/URI generation
├── models/                  # Data types
│   ├── challenge.rs
│   ├── mint.rs
│   └── tree.rs
├── db/                      # SQLite CRUD
│   ├── challenges.rs
│   ├── mints.rs
│   └── trees.rs
└── bin/
    └── test_endpoints.rs    # E2E test runner
```
