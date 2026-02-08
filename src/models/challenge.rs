use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    Arithmetic,
    ModularMath,
    LogicSequence,
    WordMath,
}

impl std::fmt::Display for ChallengeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengeType::Arithmetic => write!(f, "arithmetic"),
            ChallengeType::ModularMath => write!(f, "modular_math"),
            ChallengeType::LogicSequence => write!(f, "logic_sequence"),
            ChallengeType::WordMath => write!(f, "word_math"),
        }
    }
}

impl ChallengeType {
    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "modular_math" => ChallengeType::ModularMath,
            "logic_sequence" => ChallengeType::LogicSequence,
            "word_math" => ChallengeType::WordMath,
            _ => ChallengeType::Arithmetic,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Challenge {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub question: String,
    #[serde(skip_serializing)]
    pub answer: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing)]
    pub status: String,
}

/// Response returned to the client for GET /challenge
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub challenge_type: String,
    pub question: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub difficulty: String,
}

impl From<&Challenge> for ChallengeResponse {
    fn from(c: &Challenge) -> Self {
        ChallengeResponse {
            challenge_id: c.id.clone(),
            challenge_type: c.challenge_type.to_string(),
            question: c.question.clone(),
            expires_at: c.expires_at,
            difficulty: "medium".to_string(),
        }
    }
}
