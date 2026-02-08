use sqlx::SqlitePool;
use crate::models::challenge::{Challenge, ChallengeType};

pub async fn insert_challenge(pool: &SqlitePool, challenge: &Challenge) -> Result<(), sqlx::Error> {
    let challenge_type = challenge.challenge_type.to_string();
    let expires_at = challenge.expires_at.to_rfc3339();

    sqlx::query(
        "INSERT INTO challenges (id, challenge_type, question, answer, status, expires_at)
         VALUES (?, ?, ?, ?, 'pending', ?)"
    )
    .bind(&challenge.id)
    .bind(&challenge_type)
    .bind(&challenge.question)
    .bind(&challenge.answer)
    .bind(&expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_challenge(pool: &SqlitePool, id: &str) -> Result<Option<Challenge>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT id, challenge_type, question, answer, status, expires_at FROM challenges WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, challenge_type, question, answer, status, expires_at)| {
        Challenge {
            id,
            challenge_type: ChallengeType::from_str_loose(&challenge_type),
            question,
            answer,
            status,
            expires_at: chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }))
}

pub async fn mark_challenge_consumed(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE challenges SET status = 'consumed', consumed_at = datetime('now') WHERE id = ?"
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn expire_challenge(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE challenges SET status = 'expired' WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
