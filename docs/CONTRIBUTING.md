# Contributing to ClawPFP

Thanks for wanting to contribute! ClawPFP is open source and welcomes PRs of all sizes.

## Getting Started

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/clawpfp.git
cd clawpfp

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# 4. Set up environment
cp .env.example .env
# Edit .env with your Solana keypair and Supabase URL

# 5. Build
cargo build

# 6. Run tests
cargo run -- serve      # Terminal 1
cargo run --bin test_endpoints  # Terminal 2
```

## Project Structure

- `src/routes/` — HTTP handlers (health, challenge, mint, status)
- `src/services/` — Business logic (challenge gen, Bubblegum, Irys, tree management)
- `src/db/` — PostgreSQL queries
- `src/models/` — Request/response types
- `src/bin/test_endpoints.rs` — E2E test suite

## What to Work On

- New challenge types (puzzles, riddles, code evaluation)
- Frontend / dashboard for artists
- Multi-collection support
- Batch minting
- Mainnet deployment tooling
- Better error messages
- Documentation improvements
- Performance optimizations

## Pull Requests

1. Fork the repo
2. Create a branch (`git checkout -b feature/my-thing`)
3. Make your changes
4. Run the E2E tests to make sure nothing breaks
5. Commit with a clear message
6. Open a PR

Keep PRs focused — one feature or fix per PR.

## Code Style

- Follow existing patterns in the codebase
- Use `cargo fmt` before committing
- Use `cargo clippy` to catch common issues
- Keep functions small and focused

## Questions?

Open an issue on GitHub. No question is too small.
