use sqlx::PgPool;

pub async fn insert_mint(
    pool: &PgPool,
    asset_id: &str,
    tree_address: &str,
    leaf_index: u64,
    recipient_wallet: &str,
    metadata_uri: &str,
    metadata_name: &str,
    tx_signature: &str,
    challenge_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO mints (asset_id, tree_address, leaf_index, recipient_wallet, metadata_uri, metadata_name, tx_signature, challenge_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(asset_id)
    .bind(tree_address)
    .bind(leaf_index as i64)
    .bind(recipient_wallet)
    .bind(metadata_uri)
    .bind(metadata_name)
    .bind(tx_signature)
    .bind(challenge_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_mint_by_tx(
    pool: &PgPool,
    tx_signature: &str,
) -> Result<Option<(String, String, String, String, String)>, sqlx::Error> {
    // Returns (tx_signature, status, asset_id, recipient_wallet, created_at)
    let row = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT tx_signature, status, asset_id, recipient_wallet, created_at::text
         FROM mints WHERE tx_signature = $1"
    )
    .bind(tx_signature)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_total_minted(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mints WHERE status = 'confirmed'")
        .fetch_one(pool)
        .await?;
    Ok(count)
}
