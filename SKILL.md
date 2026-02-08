---
name: cnft-mint
version: 1.0.0
description: Mint a compressed NFT (cNFT) on Solana by solving a math/logic challenge. Server handles all signing — just provide your wallet address.
metadata: {"category":"nft","chain":"solana","compression":"bubblegum","requires":{"challenge_response":true,"solana_wallet":true,"min_sol":"0"},"cost_per_mint":"~0.000015 SOL (paid by server)"}
---

# cNFT Mint Skill

Mint a compressed NFT on Solana by solving a math or logic challenge. Each mint produces a unique pixel-art avatar stored permanently on Arweave.

## Prerequisites

- A **Solana wallet address** (public key only — no private key needed)
- Ability to solve simple math/logic challenges

## Security

- **No private key required** — the server handles all transaction signing
- You only provide your **public wallet address** to receive the cNFT
- This skill makes only HTTPS API calls. It does not access your filesystem, run shell commands, or execute arbitrary code
- Each challenge is single-use with a 5-minute expiry to prevent replay attacks

## How It Works

The mint flow has two steps: **get challenge → solve & mint**.

### Step 1: Request a challenge

```bash
curl $BASE_URL/challenge
```

Response:
```json
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "challenge_type": "arithmetic",
  "question": "What is 847 * 23 + 156?",
  "expires_at": "2026-02-08T15:05:00Z",
  "difficulty": "medium"
}
```

Challenge types:
| Type | Example |
|------|---------|
| `arithmetic` | `What is 847 * 23 + 156?` |
| `modular_math` | `What is 4^6 mod 48?` |
| `logic_sequence` | `What comes next: 2, 8, 32, 128, ?` |
| `word_math` | `What is seven hundred plus forty-two?` |

### Step 2: Solve the challenge and mint

Evaluate the challenge and send the answer with your wallet address:

```bash
curl -X POST $BASE_URL/mint \
  -H "Content-Type: application/json" \
  -d '{
    "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
    "answer": "19637",
    "wallet_address": "YOUR_SOLANA_PUBLIC_KEY"
  }'
```

Response:
```json
{
  "success": true,
  "tx_signature": "5UfD...3kQz",
  "asset_id": "BvR9...7mNp",
  "mint_index": 42,
  "message": "cNFT minted successfully"
}
```

Your cNFT is now in your wallet. Each mint gets a unique DiceBear pixel-art avatar.

## Full Example (Agent Workflow)

```python
import requests

BASE = "http://localhost:3000"  # Replace with your server URL
WALLET = "YOUR_SOLANA_PUBLIC_KEY"

# 1. Get challenge
challenge = requests.get(f"{BASE}/challenge").json()
print(f"Challenge: {challenge['question']}")

# 2. Solve it (example for arithmetic)
question = challenge["question"]
expr = question.replace("What is ", "").rstrip("?")
answer = str(eval(expr))

# 3. Mint
result = requests.post(f"{BASE}/mint", json={
    "challenge_id": challenge["challenge_id"],
    "answer": answer,
    "wallet_address": WALLET
}).json()

print(f"Minted! Asset ID: {result['asset_id']}")
print(f"Tx: https://solscan.io/tx/{result['tx_signature']}?cluster=devnet")
```

## API Reference

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Server status and tree capacity |
| GET | `/challenge` | Get a challenge to solve |
| POST | `/mint` | Submit answer + wallet to mint a cNFT |
| GET | `/status/:tx_signature` | Check mint transaction status |

### GET `/health`

**Success (200):**
```json
{
  "status": "ok",
  "active_tree": "CvNByAg...",
  "tree_capacity_remaining": 16383,
  "total_minted": 1,
  "version": "0.1.0"
}
```

### GET `/challenge`

**Success (200):**
```json
{
  "challenge_id": "string — unique ID (pass to /mint)",
  "challenge_type": "string — arithmetic|modular_math|logic_sequence|word_math",
  "question": "string — the challenge to solve",
  "expires_at": "string — ISO 8601 timestamp (5 min from now)",
  "difficulty": "string — always 'medium'"
}
```

### POST `/mint`

**Request body:**
```json
{
  "challenge_id": "string (required) — challenge ID from /challenge",
  "answer": "string (required) — your answer to the challenge",
  "wallet_address": "string (required) — Solana public key to receive the cNFT"
}
```

**Success (200):**
```json
{
  "success": true,
  "tx_signature": "string — Solana transaction signature",
  "asset_id": "string — compressed NFT asset ID",
  "mint_index": 42,
  "message": "cNFT minted successfully"
}
```

### GET `/status/:tx_signature`

**Success (200):**
```json
{
  "tx_signature": "string — the transaction signature",
  "status": "string — confirmed|not_found",
  "asset_id": "string | null",
  "recipient": "string | null",
  "confirmed_at": "string | null — ISO 8601 timestamp"
}
```

## Error Codes

### `/challenge`

| Code | Meaning |
|------|---------|
| 500 | Server error |

### `/mint`

| Code | Meaning |
|------|---------|
| 400 | Invalid wallet address, missing fields, incorrect answer, or challenge already used |
| 410 | Challenge expired |
| 500 | Server error (tree creation, Arweave upload, or Solana transaction failed) |

### `/status/:tx_signature`

| Code | Meaning |
|------|---------|
| 200 | Always returns — check `status` field for `confirmed` or `not_found` |

## Notes

- **No signing required** — the server pays all fees and signs the transaction
- **Compressed NFTs** — uses Metaplex Bubblegum for ~1000x cheaper mints than regular NFTs
- **Permanent metadata** — NFT metadata and image URL stored on Arweave via Irys
- **Unique avatars** — each mint gets a unique DiceBear pixel-art avatar
- **Stateless** — no session or login required
- **Challenge expiration** — challenges expire after 5 minutes
- **Single-use challenges** — each challenge ID can only be used once
- **Auto-scaling** — Merkle trees hold 16,384 cNFTs each and rotate automatically
- **Cost** — ~0.000015 SOL per mint (paid by the server, free for the agent)
