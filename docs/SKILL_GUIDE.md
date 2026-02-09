# Writing a SKILL.md for Your ClawPFP Deployment

When you fork ClawPFP and deploy your own collection, you need a `SKILL.md` file that tells AI agents how to interact with your server. This guide explains the format, every section, and how to customize it for your collection.

Your `SKILL.md` is served at `https://your-domain.com/skill.md` and is the single source of truth agents use to discover and interact with your mint API.

## What is a SKILL.md?

A SKILL.md is a markdown file with a YAML frontmatter header that describes an API skill for AI agents. Think of it as a machine-readable API reference — agents read this file to learn:

- What your API does
- What endpoints are available
- What request/response formats to use
- What errors to expect
- Security guarantees

## File Structure

A SKILL.md has two parts:

1. **YAML frontmatter** — Machine-readable metadata between `---` delimiters
2. **Markdown body** — Human/agent-readable documentation

## Part 1: YAML Frontmatter

The frontmatter is the most important part. It's what agents parse first to decide whether to use your skill.

```yaml
---
name: your-collection-name
version: 1.0.0
description: One-line description of what agents can do with your API.
metadata: {"category":"nft","api_base":"https://your-domain.com","chain":"solana","compression":"bubblegum","requires":{"challenge_response":true,"solana_wallet":true,"min_sol":"0"},"cost_per_mint":"~0.000015 SOL (paid by server)"}
---
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Short, lowercase identifier for your skill. Use your collection or project name. No spaces — use hyphens. |
| `version` | Yes | Semantic version (`1.0.0`). Bump when you change the API. |
| `description` | Yes | One sentence describing what the agent can do. Be specific — mention "cNFT", "Solana", "challenge-response". |
| `metadata` | Yes | JSON object with structured info agents can parse programmatically. |

### Metadata Object

The `metadata` field is a JSON object (written inline). Here's what to include:

```json
{
  "category": "nft",
  "api_base": "https://your-domain.com",
  "chain": "solana",
  "compression": "bubblegum",
  "requires": {
    "challenge_response": true,
    "solana_wallet": true,
    "min_sol": "0"
  },
  "cost_per_mint": "~0.000015 SOL (paid by server)"
}
```

| Key | Description |
|-----|-------------|
| `category` | What kind of skill this is. Use `"nft"` for ClawPFP. |
| `api_base` | Your deployed server URL. No trailing slash. |
| `chain` | Blockchain. Always `"solana"` for ClawPFP. |
| `compression` | NFT compression method. Always `"bubblegum"` for ClawPFP. |
| `requires.challenge_response` | `true` — agents must solve a challenge before minting. |
| `requires.solana_wallet` | `true` — agents need a Solana public key. |
| `requires.min_sol` | `"0"` — the server pays all fees, agents need zero SOL. |
| `cost_per_mint` | Human-readable cost string. Helps agents decide whether to mint. |

## Part 2: Markdown Body

The body follows a specific structure. Here's each section and what to write:

### Title & Introduction

```markdown
# cNFT Mint Skill

Mint a compressed NFT on Solana by solving a math or logic challenge.
Each mint produces a unique pixel-art avatar stored permanently on Arweave.
```

Keep it to 1-2 sentences. Mention what makes your collection unique (avatar style, theme, etc.).

### Prerequisites

```markdown
## Prerequisites

- A **Solana wallet address** (public key only — no private key needed)
- Ability to solve simple math/logic challenges
```

List exactly what an agent needs before it can use your API. For ClawPFP, this is always a wallet address and challenge-solving ability.

### Security

```markdown
## Security

- **No private key required** — the server handles all transaction signing
- You only provide your **public wallet address** to receive the cNFT
- This skill makes only HTTPS API calls. It does not access your filesystem, run shell commands, or execute arbitrary code
- Each challenge is single-use with a 5-minute expiry to prevent replay attacks
```

This section builds agent trust. Always include:
- No private key needed
- Only public wallet address required
- HTTPS only, no filesystem/shell access
- Challenge replay protection

### How It Works

This is the core section. Walk through the complete mint flow step by step with `curl` examples and response bodies.

```markdown
## How It Works

The mint flow has two steps: **get challenge → solve & mint**.

### Step 1: Request a challenge

\```bash
curl https://your-domain.com/challenge
\```

Response:
\```json
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "challenge_type": "arithmetic",
  "question": "What is 847 * 23 + 156?",
  "expires_at": "2026-02-08T15:05:00Z",
  "difficulty": "medium"
}
\```
```

**Important:** Replace `your-domain.com` with your actual deployed URL everywhere.

Include a challenge type table so agents know what to expect:

```markdown
Challenge types:
| Type | Example |
|------|---------|
| `arithmetic` | `What is 847 * 23 + 156?` |
| `modular_math` | `What is 4^6 mod 48?` |
| `logic_sequence` | `What comes next: 2, 8, 32, 128, ?` |
| `word_math` | `What is seven hundred plus forty-two?` |
```

Then document Step 2 (the mint call) with a full `curl` example and the success response.

### Full Example

Include a complete Python example that agents can copy and adapt:

```markdown
## Full Example (Agent Workflow)

\```python
import requests

BASE = "https://your-domain.com"
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
\```
```

This is the most important section for agents — a working code snippet they can execute.

### API Reference

Document every endpoint with method, path, request/response schemas, and field descriptions:

```markdown
## API Reference

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Server status and tree capacity |
| GET | `/challenge` | Get a challenge to solve |
| POST | `/mint` | Submit answer + wallet to mint a cNFT |
| GET | `/status/:tx_signature` | Check mint transaction status |
```

Then expand each endpoint with its full request/response JSON. Use descriptive field annotations like:

```json
{
  "challenge_id": "string — unique ID (pass to /mint)",
  "challenge_type": "string — arithmetic|modular_math|logic_sequence|word_math",
  "question": "string — the challenge to solve"
}
```

### Error Codes

Document error codes per endpoint so agents can handle failures:

```markdown
## Error Codes

### `/mint`

| Code | Meaning |
|------|---------|
| 400 | Invalid wallet address, missing fields, incorrect answer, or challenge already used |
| 410 | Challenge expired |
| 500 | Server error |
```

### Notes

End with operational details agents should know:

```markdown
## Notes

- **No signing required** — the server pays all fees
- **Compressed NFTs** — uses Metaplex Bubblegum for ~1000x cheaper mints
- **Permanent metadata** — stored on Arweave via Irys
- **Challenge expiration** — challenges expire after 5 minutes
- **Single-use challenges** — each challenge ID can only be used once
- **Auto-scaling** — Merkle trees rotate automatically when full
- **Cost** — ~0.000015 SOL per mint (paid by the server, free for the agent)
```

## Complete Template

Here's a minimal SKILL.md you can copy and customize:

```markdown
---
name: your-collection
version: 1.0.0
description: Mint a compressed NFT (cNFT) on Solana by solving a math/logic challenge. Server handles all signing — just provide your wallet address.
metadata: {"category":"nft","api_base":"https://your-domain.com","chain":"solana","compression":"bubblegum","requires":{"challenge_response":true,"solana_wallet":true,"min_sol":"0"},"cost_per_mint":"~0.000015 SOL (paid by server)"}
---

# cNFT Mint Skill

Mint a compressed NFT on Solana by solving a challenge. Each mint produces
a unique avatar stored permanently on Arweave.

## Prerequisites

- A **Solana wallet address** (public key only — no private key needed)
- Ability to solve simple math/logic challenges

## Security

- **No private key required** — the server handles all transaction signing
- You only provide your **public wallet address** to receive the cNFT
- This skill makes only HTTPS API calls
- Each challenge is single-use with a 5-minute expiry

## How It Works

### Step 1: Request a challenge

```bash
curl https://your-domain.com/challenge
```

Response:
```json
{
  "challenge_id": "uuid",
  "challenge_type": "arithmetic",
  "question": "What is 847 * 23 + 156?",
  "expires_at": "2026-02-08T15:05:00Z",
  "difficulty": "medium"
}
```

### Step 2: Solve and mint

```bash
curl -X POST https://your-domain.com/mint \
  -H "Content-Type: application/json" \
  -d '{
    "challenge_id": "uuid-from-step-1",
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

## API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Server status and tree capacity |
| GET | `/challenge` | Get a challenge to solve |
| POST | `/mint` | Submit answer + wallet to mint |
| GET | `/status/:tx_signature` | Check transaction status |

## Error Codes

| Endpoint | Code | Meaning |
|----------|------|---------|
| `/challenge` | 500 | Server error |
| `/mint` | 400 | Bad request (wrong answer, invalid wallet, used challenge) |
| `/mint` | 410 | Challenge expired |
| `/mint` | 500 | Server error |

## Notes

- No signing required — server pays all fees
- Compressed NFTs via Metaplex Bubblegum
- Permanent metadata on Arweave
- Challenges expire after 5 minutes and are single-use
- ~0.000015 SOL per mint (paid by server, free for agents)
```

## Customization Checklist

When creating your own SKILL.md, update these items:

- [ ] `name` in frontmatter — your collection/project name
- [ ] `api_base` in metadata — your deployed URL
- [ ] `description` — what makes your collection unique
- [ ] All `curl` URLs — replace with your domain
- [ ] All example URLs in Python snippet — replace with your domain
- [ ] Collection-specific details (avatar style, theme, etc.)
- [ ] Any custom challenge types you've added
- [ ] Cost per mint if you've changed tree configuration

## Tips

1. **Be precise** — agents parse this literally. Incorrect field names or URLs cause failures.
2. **Include working examples** — agents rely heavily on the Python snippet.
3. **Document all error codes** — agents need to handle failures gracefully.
4. **Keep it current** — update the SKILL.md whenever you change the API.
5. **Test it** — visit `https://your-domain.com/skill.md` in a browser to verify it's served correctly.
6. **No trailing slashes** — use `https://your-domain.com` not `https://your-domain.com/`.
