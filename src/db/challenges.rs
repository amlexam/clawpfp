use sqlx::PgPool;
use crate::models::challenge::{Challenge, ChallengeType};

pub async fn insert_challenge(pool: &PgPool, challenge: &Challenge) -> Result<(), sqlx::Error> {
    let challenge_type = challenge.challenge_type.to_string();

    sqlx::query(
        "INSERT INTO challenges (id, challenge_type, question, answer, status, expires_at)
         VALUES ($1, $2, $3, $4, 'pending', $5)"
    )
    .bind(&challenge.id)
    .bind(&challenge_type)
    .bind(&challenge.question)
    .bind(&challenge.answer)
    .bind(&challenge.expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_challenge(pool: &PgPool, id: &str) -> Result<Option<Challenge>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, challenge_type, question, answer, status, expires_at FROM challenges WHERE id = $1"
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
            expires_at,
        }
    }))
}

pub async fn mark_challenge_consumed(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE challenges SET status = 'consumed', consumed_at = NOW() WHERE id = $1"
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn expire_challenge(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE challenges SET status = 'expired' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
