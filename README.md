<div align="center">

# ClawPFP

<img src="https://media1.tenor.com/m/cOxR1hF63Y0AAAAd/matrix-film.gif" width="600" />

**Open-source infrastructure for artists to host NFT collections mintable by AI agents on Solana — for under $10.**

</div>

ClawPFP handles everything: challenge-based bot verification, metadata creation, permanent Arweave storage, and compressed NFT minting. Artists configure their collection once, deploy, and any AI agent can mint by solving a math puzzle. Each NFT gets a unique generative avatar.

**Cost:** ~$0.002 per mint. A 1,000-piece collection costs under $10 total.

**Live demo:** [api.clawpfp.com/health](https://api.clawpfp.com/health) | **Skill file:** [api.clawpfp.com/skill.md](https://api.clawpfp.com/skill.md)

## How It Works

```
Agent                        ClawPFP                         Solana
  |                             |                               |
  |  GET /challenge             |                               |
  |  ────────────────────────>  |  generate math puzzle         |
  |  { question }              |                               |
  |  <────────────────────────  |                               |
  |                             |                               |
  |  POST /mint                 |                               |
  |  { answer, wallet }         |                               |
  |  ────────────────────────>  |  verify ──> upload ──> mint   |
  |                             |           Arweave    Solana   |
  |  { tx_signature, asset_id } |                               |
  |  <────────────────────────  |                               |
```

Two API calls. No wallet signing. No private keys. The agent just needs a public address.

## Quick Start

```bash
# Clone and build
git clone https://github.com/aarjn/clawpfp.git && cd clawpfp
cargo build

# Configure
cp .env.example .env
# Edit .env — set PAYER_KEYPAIR, SOLANA_RPC_URL, DATABASE_URL

# Create your collection (one-time)
cargo run -- setup
# Copy the printed COLLECTION_MINT into .env

# Fund Irys for metadata uploads
solana transfer 4a7s9iC5NwfUtf8fXpKWxYXcekfqiN6mRqipYXMtcrUS 0.01 \
  --url https://api.devnet.solana.com --keypair <your-keypair>

# Launch
cargo run -- serve
```

Your server is live at `http://localhost:3000`. Agents can now mint from your collection.

## For Artists: Configure Your Collection

Edit `.env` to customize your NFTs:

```bash
COLLECTION_NAME=Dreamer                    # NFT names: "Dreamer #0", "Dreamer #1", ...
COLLECTION_SYMBOL=DRM                      # Token symbol
COLLECTION_DESCRIPTION="Your description"  # Embedded in every NFT's metadata
SELLER_FEE_BASIS_POINTS=500                # Royalty: 500 = 5%

# Each mint generates a unique avatar via DiceBear — customize colors, accessories, etc.
COLLECTION_IMAGE_URL=https://api.dicebear.com/9.x/pixel-art/svg?seed=cnft-{mint_index}&backgroundColor=6c5ce7,00b894
```

See [.env.example](.env.example) for all options with detailed explanations.

## For Agents: Mint an NFT

```python
import requests

BASE = "https://api.clawpfp.com"

# 1. Get a challenge
challenge = requests.get(f"{BASE}/challenge").json()

# 2. Solve it
answer = str(eval(challenge["question"].replace("What is ", "").rstrip("?")))

# 3. Mint
result = requests.post(f"{BASE}/mint", json={
    "challenge_id": challenge["challenge_id"],
    "answer": answer,
    "wallet_address": "YOUR_SOLANA_PUBKEY"
}).json()

print(f"Minted: {result['asset_id']}")
```

Full API documentation: [SKILL.md](SKILL.md)

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Server status and remaining capacity |
| GET | `/challenge` | Get a math/logic puzzle (4 types, 5-min expiry) |
| POST | `/mint` | Solve challenge + provide wallet = get a cNFT |
| GET | `/status/:tx` | Check transaction confirmation |
| GET | `/skill.md` | Agent-readable API documentation |

## Deploy to Production

ClawPFP includes a Dockerfile and Railway config. One-click deploy:

1. Push to GitHub
2. Connect repo in [Railway](https://railway.app)
3. Add your `.env` variables in Railway's Variables tab
4. Deploy

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for other deployment options (Fly.io, Docker, bare metal).

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| **Rust** | [Install](https://rustup.rs) |
| **Solana CLI** | [Install](https://docs.solana.com/cli/install-solana-cli-tools) |
| **Solana keypair** | `solana-keygen new` — needs ~1 SOL on devnet |
| **Supabase** | Free tier — [supabase.com](https://supabase.com) |
| **Irys funding** | 0.01 SOL covers ~900 mints |

## Cost Breakdown

| Item | Cost |
|------|------|
| Merkle tree (16,384 capacity) | ~0.68 SOL (one-time) |
| Per-mint transaction | ~0.000005 SOL |
| Per-mint Arweave upload | ~0.00001 SOL |
| **Total per mint** | **~$0.002** |
| **1,000-piece collection** | **< $10** |

## Testing

```bash
# Start server
cargo run -- serve

# Run the 8-step E2E test suite
cargo run --bin test_endpoints

# Test against production
TEST_BASE_URL=https://api.clawpfp.com cargo run --bin test_endpoints
```

## Documentation

| Document | Description |
|----------|-------------|
| [SKILL.md](SKILL.md) | Agent-facing API reference with examples |
| [docs/SKILL_GUIDE.md](docs/SKILL_GUIDE.md) | How to write your own SKILL.md |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, database schema, security model |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | How to contribute |
| [docs/LICENSE.md](docs/LICENSE.md) | License |
| [.env.example](.env.example) | All configuration options with explanations |

## Contributing

PRs welcome! See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md).

## License

See [./LICENSE.md](./LICENSE.md).
