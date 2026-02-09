# ClawPFP Architecture

## Overview

ClawPFP is an open-source Solana infrastructure that lets any artist host an NFT collection mintable by AI agents. The server handles everything: challenge generation, metadata creation, Arweave uploads, and on-chain minting. Agents only need a wallet address and the ability to solve a math problem.

**Stack:** Rust + Axum + Solana + Metaplex Bubblegum + Arweave (via Irys) + PostgreSQL (Supabase)

## System Flow

```
Agent                            ClawPFP Server                     External Services
  |                                   |                                    |
  |  1. GET /challenge                |                                    |
  |---------------------------------->|                                    |
  |                                   |  Generate random math puzzle       |
  |                                   |  Store in PostgreSQL               |
  |  { challenge_id, question }       |                                    |
  |<----------------------------------|                                    |
  |                                   |                                    |
  |  2. Solve: "What is 847 * 23?"    |                                    |
  |     Answer: "19481"               |                                    |
  |                                   |                                    |
  |  3. POST /mint                    |                                    |
  |  { answer, wallet_address }       |                                    |
  |---------------------------------->|                                    |
  |                                   |  a. Verify answer + expiry         |
  |                                   |  b. Mark challenge consumed        |
  |                                   |                                    |
  |                                   |  c. Build Metaplex JSON            |
  |                                   |     (name, image, attributes)      |
  |                                   |                                    |
  |                                   |  d. Sign ANS-104 data item ------->| Irys/Arweave
  |                                   |     Upload metadata               | (permanent storage)
  |                                   |     Get arweave:// URI      <------|
  |                                   |                                    |
  |                                   |  e. Build Bubblegum tx ----------->| Solana
  |                                   |     MintToCollectionV1             | (append leaf to
  |                                   |     Sign + send + confirm   <------| Merkle tree)
  |                                   |                                    |
  |                                   |  f. Record mint in PostgreSQL      |
  |  { tx_signature, asset_id }       |                                    |
  |<----------------------------------|                                    |
```

## Component Architecture

```
                    ┌─────────────────────────────────────────────┐
                    │              Axum HTTP Server                │
                    │         (CORS + Tracing + Rate Limit)        │
                    ├────────┬──────────┬──────────┬──────────────┤
                    │/health │/challenge│  /mint   │/status/:tx   │
                    │        │          │          │/skill.md     │
                    └───┬────┴────┬─────┴────┬─────┴──────┬───────┘
                        │         │          │            │
                        │         ▼          ▼            │
                        │   ┌──────────┐ ┌──────────────┐ │
                        │   │Challenge │ │ Mint Pipeline │ │
                        │   │Generator │ │              │ │
                        │   │          │ │ 1. Validate  │ │
                        │   │ 4 types: │ │ 2. Metadata  │ │
                        │   │ arith    │ │ 3. Irys      │ │
                        │   │ modular  │ │ 4. Bubblegum │ │
                        │   │ sequence │ │ 5. Record    │ │
                        │   │ word     │ │              │ │
                        │   └────┬─────┘ └─┬──┬──┬──┬──┘ │
                        │        │         │  │  │  │    │
                        ▼        ▼         ▼  │  │  ▼    ▼
                    ┌──────────────────────────┤──┤────────────┐
                    │    PostgreSQL (Supabase)  │  │            │
                    │                          │  │            │
                    │  challenges  mints  merkle_trees         │
                    └──────────────────────────┘──┘────────────┘
                                               │  │
                              ┌────────────────┘  └────────────┐
                              ▼                                ▼
                    ┌──────────────────┐            ┌──────────────────┐
                    │  Irys / Arweave  │            │      Solana      │
                    │                  │            │                  │
                    │  - ANS-104 items │            │  - Bubblegum     │
                    │  - Avro tags     │            │    program       │
                    │  - SHA-384 hash  │            │  - SPL Account   │
                    │  - ed25519 sign  │            │    Compression   │
                    │                  │            │  - Merkle trees  │
                    │  Permanent       │            │  - cNFTs as      │
                    │  metadata JSON   │            │    tree leaves   │
                    └──────────────────┘            └──────────────────┘
```

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Language** | Rust | Performance, safety, Solana SDK support |
| **Web framework** | Axum 0.8 | Async, tower middleware, type-safe extractors |
| **Database** | PostgreSQL (Supabase) | Free tier, managed, connection pooling |
| **ORM** | sqlx 0.8 | Compile-time checked queries, async, migrations |
| **Blockchain** | Solana (solana-sdk 2.1) | Fast, cheap, cNFT support |
| **NFT program** | Metaplex Bubblegum | Compressed NFT standard |
| **Compression** | SPL Account Compression | Merkle tree state management |
| **Metadata** | Arweave via Irys | Permanent, decentralized storage |
| **Images** | DiceBear Pixel Art API | Unique avatar per mint, no hosting needed |
| **Rate limiting** | tower-governor | Per-IP rate limiting middleware |

## Project Structure

```
clawpfp/
├── Cargo.toml                    # Dependencies and project config
├── Dockerfile                    # Multi-stage build for Railway
├── railway.toml                  # Railway deployment config
├── rust-toolchain.toml           # Rust version pinning
├── SKILL.md                      # Agent-facing API documentation
├── migrations/
│   └── 001_initial.sql           # PostgreSQL schema
├── docs/
│   ├── ARCHITECTURE.md           # This file
│   ├── SKILL_GUIDE.md            # How to write your own SKILL.md
│   ├── CONTRIBUTING.md           # Contribution guide
│   └── LICENSE.md                # License
├── src/
│   ├── main.rs                   # Entry point, CLI routing, server startup
│   ├── config.rs                 # Environment variable loading + validation
│   ├── state.rs                  # Shared application state (AppState)
│   ├── error.rs                  # Error types → HTTP response mapping
│   ├── setup.rs                  # One-time collection NFT creation
│   ├── routes/
│   │   ├── mod.rs                # Router definition + /skill.md serving
│   │   ├── health.rs             # GET /health — status + capacity
│   │   ├── challenge.rs          # GET /challenge — generate puzzle
│   │   ├── mint.rs               # POST /mint — full mint pipeline
│   │   └── status.rs             # GET /status/:tx — tx lookup
│   ├── services/
│   │   ├── mod.rs                # Service module exports
│   │   ├── challenge.rs          # 4 challenge generators + verifier
│   │   ├── bubblegum.rs          # Bubblegum instruction builders + PDA derivation
│   │   ├── tree_manager.rs       # Merkle tree lifecycle (create, rotate, track)
│   │   ├── irys.rs               # ANS-104 data items + Arweave upload
│   │   ├── metadata.rs           # Metaplex JSON builder with DiceBear URLs
│   │   └── solana.rs             # RPC helper functions
│   ├── models/
│   │   ├── mod.rs                # Model module exports
│   │   ├── challenge.rs          # Challenge + ChallengeResponse types
│   │   ├── mint.rs               # MintRequest + MintResponse types
│   │   └── tree.rs               # TreeInfo + TreeRow types
│   ├── db/
│   │   ├── mod.rs                # Database module exports
│   │   ├── challenges.rs         # Challenge CRUD (insert, get, consume, expire)
│   │   ├── mints.rs              # Mint record insertion + lookup
│   │   └── trees.rs              # Tree tracking (insert, deactivate, increment)
│   └── bin/
│       └── test_endpoints.rs     # 8-step end-to-end test suite
└── .env.example                  # Documented environment template
```

## Database Schema

Three tables, auto-created via `migrations/001_initial.sql` on startup:

### `challenges`
Tracks challenge-response flow. Prevents replay attacks.

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | UUID v4 |
| `challenge_type` | TEXT | arithmetic, modular_math, logic_sequence, word_math |
| `question` | TEXT | The puzzle shown to the agent |
| `answer` | TEXT | Correct answer (never exposed via API) |
| `status` | TEXT | `pending` → `consumed` or `expired` |
| `created_at` | TIMESTAMPTZ | Auto-set |
| `expires_at` | TIMESTAMPTZ | 5 minutes from creation |
| `consumed_at` | TIMESTAMPTZ | Set when used for a successful mint |

### `mints`
Audit trail of every minted cNFT.

| Column | Type | Description |
|--------|------|-------------|
| `id` | BIGSERIAL PK | Auto-increment |
| `asset_id` | TEXT UNIQUE | Derived Bubblegum asset ID |
| `tree_address` | TEXT FK | Merkle tree this leaf belongs to |
| `leaf_index` | BIGINT | Position in the Merkle tree |
| `recipient_wallet` | TEXT | Solana address that received the cNFT |
| `metadata_uri` | TEXT | Permanent Arweave URL |
| `metadata_name` | TEXT | e.g. "Dreamer #42" |
| `tx_signature` | TEXT | Solana transaction signature |
| `challenge_id` | TEXT FK | Which challenge was solved |
| `status` | TEXT | `confirmed` |
| `created_at` | TIMESTAMPTZ | Auto-set |

### `merkle_trees`
Tracks active Merkle trees and handles automatic rotation.

| Column | Type | Description |
|--------|------|-------------|
| `id` | BIGSERIAL PK | Auto-increment |
| `address` | TEXT UNIQUE | Solana pubkey of the tree account |
| `max_depth` | INTEGER | Tree depth (14 = 16,384 capacity) |
| `max_buffer_size` | INTEGER | Concurrent update buffer size |
| `canopy_depth` | INTEGER | On-chain proof cache depth |
| `max_capacity` | BIGINT | Total leaves (2^max_depth) |
| `current_leaf_index` | BIGINT | Next available slot |
| `is_active` | BOOLEAN | Only one tree active at a time |
| `creation_tx` | TEXT | Tree creation transaction signature |
| `created_at` | TIMESTAMPTZ | Auto-set |

## Challenge System

Four challenge types are generated randomly with equal probability:

| Type | Example | How it works |
|------|---------|-------------|
| **Arithmetic** | `What is 196 - 41 * 848?` | 3 random numbers + 2 random operators, standard math precedence |
| **Modular math** | `What is 4^6 mod 48?` | Modular exponentiation via square-and-multiply |
| **Logic sequence** | `What comes next: 6, 18, 54, 162, ?` | Geometric sequence with random start + ratio |
| **Word math** | `Sum of letter values in 'SOLANA'?` | A=1, B=2...Z=26, word from [SOLANA, MINT, AGENT, CHAIN, TOKEN, BLOCK] |

Challenges are single-use, expire after 5 minutes, and are cleaned up by a background task every 60 seconds.

## Irys/Arweave Upload (ANS-104)

The server implements the ANS-104 data item spec from scratch:

1. **Build tags** — Content-Type and other metadata encoded as Apache Avro binary (zigzag varint lengths + array terminator)
2. **Deep hash** — SHA-384 recursive hash of data item fields
3. **Sign** — ed25519 signature over the deep hash
4. **Assemble** — Binary data item: signature + owner pubkey + tags + data
5. **Upload** — POST to Irys node, returns permanent `https://arweave.net/{id}` URL

## Merkle Tree Management

- **Auto-creation** — First mint creates a tree (~0.68 SOL, 16,384 leaf capacity)
- **Leaf tracking** — `current_leaf_index` incremented after each mint
- **Rotation** — When a tree fills up, it's deactivated and a new one is created
- **Single active tree** — Only one tree is active at any time

## Security

| Measure | Implementation |
|---------|---------------|
| **No client signing** | Server holds the keypair, fully signs all transactions |
| **Challenge replay protection** | Each challenge_id is consumed after one use |
| **Challenge expiry** | 5-minute TTL, background cleanup task |
| **Rate limiting** | tower-governor per-IP rate limiting |
| **Keypair isolation** | Loaded from env var or base64, never logged or exposed |
| **Input validation** | Wallet address parsed as Solana Pubkey, challenge verified before mint |

## Cost Breakdown

| Item | Cost | Frequency |
|------|------|-----------|
| Merkle tree creation | ~0.68 SOL (~$100) | Once per 16,384 mints |
| Mint transaction fee | ~0.000005 SOL | Per mint |
| Irys metadata upload | ~0.00001 SOL | Per mint |
| **Cost per mint** | **~0.000015 SOL (~$0.002)** | After tree exists |
| **16,384 mints total** | **~0.93 SOL (~$135)** | Tree + all mints |

For an artist launching a 1,000-piece collection: **under $10 total**.

## Deployment

The server compiles to a single binary. Minimum requirements: 1 vCPU, 512MB RAM.

Supported deployment targets:
- **Railway** — Dockerfile + railway.toml included, auto-deploys from GitHub
- **Fly.io** — Use the Dockerfile
- **Any Docker host** — `docker build -t clawpfp . && docker run -p 3000:3000 clawpfp`
- **Bare metal** — `cargo build --release && ./target/release/clawpfp serve`
