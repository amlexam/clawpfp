# Crate: clawpfp (bin)
Edition: 2021
Version: 0.1.0
External deps: anyhow, axum, base64, borsh, bs58, chrono, dotenvy, ed25519-dalek, mpl-bubblegum, rand, regex, reqwest, serde, serde_json, sha2, solana-client, solana-sdk, spl-account-compression, spl-noop, sqlx, thiserror, tokio, tower, tower-http, tower_governor, tracing, tracing-subscriber, uuid

## Module Tree
- crate
  - config
  - db
    - challenges
    - mints
    - trees
  - error
  - models
    - challenge
    - mint
    - tree
  - routes
    - health
    - challenge
    - mint
    - status
  - services
    - challenge
    - bubblegum
    - tree_manager
    - solana
    - metadata
    - irys — Upload data to Arweave via Irys using ANS-104 signed data items.
  - setup
  - state

# Crate: test_endpoints (bin)
Edition: 2021
Version: 0.1.0
External deps: anyhow, axum, base64, borsh, bs58, chrono, dotenvy, ed25519-dalek, mpl-bubblegum, rand, regex, reqwest, serde, serde_json, sha2, solana-client, solana-sdk, spl-account-compression, spl-noop, sqlx, thiserror, tokio, tower, tower-http, tower_governor, tracing, tracing-subscriber, uuid

## Module Tree
- crate — End-to-end test runner for clawpfp.

