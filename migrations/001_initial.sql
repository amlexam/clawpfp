-- Merkle tree tracking
CREATE TABLE IF NOT EXISTS merkle_trees (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    address     TEXT NOT NULL UNIQUE,
    max_depth   INTEGER NOT NULL,
    max_buffer_size INTEGER NOT NULL,
    canopy_depth    INTEGER NOT NULL,
    max_capacity    INTEGER NOT NULL,
    current_leaf_index INTEGER NOT NULL DEFAULT 0,
    collection_mint TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    creation_tx TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Challenge state machine: pending → consumed | expired
CREATE TABLE IF NOT EXISTS challenges (
    id              TEXT PRIMARY KEY,
    challenge_type  TEXT NOT NULL,
    question        TEXT NOT NULL,
    answer          TEXT NOT NULL,
    wallet_address  TEXT,
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
    status          TEXT NOT NULL DEFAULT 'confirmed',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (tree_address) REFERENCES merkle_trees(address),
    FOREIGN KEY (challenge_id) REFERENCES challenges(id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_challenges_status ON challenges(status, expires_at);
CREATE INDEX IF NOT EXISTS idx_mints_recipient ON mints(recipient_wallet);
CREATE INDEX IF NOT EXISTS idx_mints_tree ON mints(tree_address);
CREATE INDEX IF NOT EXISTS idx_trees_active ON merkle_trees(is_active);
