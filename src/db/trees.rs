use sqlx::SqlitePool;
use crate::models::tree::TreeRow;

pub async fn get_active_tree(pool: &SqlitePool) -> Result<Option<TreeRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, String, i64, i64, i64, i64, i64, bool)>(
        "SELECT id, address, max_depth, max_buffer_size, canopy_depth, max_capacity, current_leaf_index, is_active
         FROM merkle_trees WHERE is_active = TRUE LIMIT 1"
    )
    .bind(true)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, address, max_depth, max_buffer_size, canopy_depth, max_capacity, current_leaf_index, is_active)| {
        TreeRow {
            id,
            address,
            max_depth,
            max_buffer_size,
            canopy_depth,
            max_capacity,
            current_leaf_index,
            is_active,
        }
    }))
}

pub async fn insert_tree(
    pool: &SqlitePool,
    address: &str,
    max_depth: u32,
    max_buffer_size: u32,
    canopy_depth: u32,
    max_capacity: u64,
    collection_mint: Option<&str>,
    creation_tx: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO merkle_trees (address, max_depth, max_buffer_size, canopy_depth, max_capacity, collection_mint, creation_tx)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(address)
    .bind(max_depth as i64)
    .bind(max_buffer_size as i64)
    .bind(canopy_depth as i64)
    .bind(max_capacity as i64)
    .bind(collection_mint)
    .bind(creation_tx)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn deactivate_tree(pool: &SqlitePool, address: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE merkle_trees SET is_active = FALSE WHERE address = ?")
        .bind(address)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn increment_tree_leaf_index(pool: &SqlitePool, address: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE merkle_trees SET current_leaf_index = current_leaf_index + 1 WHERE address = ?")
        .bind(address)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_tree_capacity_remaining(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT max_capacity, current_leaf_index FROM merkle_trees WHERE is_active = TRUE LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(cap, idx)| cap - idx).unwrap_or(0))
}
