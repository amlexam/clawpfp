use rand::Rng;
use uuid::Uuid;
use crate::models::challenge::{Challenge, ChallengeType};

pub fn generate_challenge(expiry_seconds: i64) -> Challenge {
    let mut rng = rand::thread_rng();
    let challenge_type = match rng.gen_range(0..4) {
        0 => ChallengeType::Arithmetic,
        1 => ChallengeType::ModularMath,
        2 => ChallengeType::LogicSequence,
        _ => ChallengeType::WordMath,
    };

    let (question, answer) = match challenge_type {
        ChallengeType::Arithmetic => generate_arithmetic(&mut rng),
        ChallengeType::ModularMath => generate_modular_math(&mut rng),
        ChallengeType::LogicSequence => generate_logic_sequence(&mut rng),
        ChallengeType::WordMath => generate_word_math(&mut rng),
    };

    Challenge {
        id: Uuid::new_v4().to_string(),
        challenge_type,
        question,
        answer,
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(expiry_seconds),
        status: "pending".to_string(),
    }
}

pub fn verify_challenge_answer(challenge: &Challenge, submitted: &str) -> bool {
    challenge.answer.trim() == submitted.trim()
}

fn generate_arithmetic(rng: &mut impl Rng) -> (String, String) {
    let a = rng.gen_range(100..999) as i64;
    let b = rng.gen_range(10..99) as i64;
    let c = rng.gen_range(10..999) as i64;
    let ops = ["+", "-", "*"];
    let op1_idx = rng.gen_range(0..3);
    let op2_idx = rng.gen_range(0..3);
    let op1 = ops[op1_idx];
    let op2 = ops[op2_idx];

    // Evaluate left to right (no precedence, as presented to the agent)
    // Actually, math precedence applies: * before +/-
    // Let's compute with standard math precedence
    let result = eval_expression(a, op1, b, op2, c);

    (
        format!("What is {} {} {} {} {}?", a, op1, b, op2, c),
        result.to_string(),
    )
}

fn eval_expression(a: i64, op1: &str, b: i64, op2: &str, c: i64) -> i64 {
    // Standard math precedence
    match (op1, op2) {
        ("*", _) => {
            let left = a * b;
            apply_op(left, op2, c)
        }
        (_, "*") => {
            let right = b * c;
            apply_op(a, op1, right)
        }
        _ => {
            let left = apply_op(a, op1, b);
            apply_op(left, op2, c)
        }
    }
}

fn apply_op(a: i64, op: &str, b: i64) -> i64 {
    match op {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        _ => 0,
    }
}

fn generate_modular_math(rng: &mut impl Rng) -> (String, String) {
    let base: u64 = rng.gen_range(2..10);
    let exp: u64 = rng.gen_range(5..20);
    let modulus: u64 = rng.gen_range(7..50);
    let result = mod_pow(base, exp, modulus);
    (
        format!("What is {}^{} mod {}?", base, exp, modulus),
        result.to_string(),
    )
}

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u64 = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % modulus;
        }
        exp /= 2;
        base = base * base % modulus;
    }
    result
}

fn generate_logic_sequence(rng: &mut impl Rng) -> (String, String) {
    let a = rng.gen_range(1..10) as i64;
    let r = rng.gen_range(2..5) as i64;
    let seq: Vec<i64> = (0..4).map(|i| a * r.pow(i)).collect();
    let next = a * r.pow(4);
    (
        format!(
            "What comes next in the sequence: {}, {}, {}, {}, ?",
            seq[0], seq[1], seq[2], seq[3]
        ),
        next.to_string(),
    )
}

fn generate_word_math(rng: &mut impl Rng) -> (String, String) {
    let words = ["SOLANA", "MINT", "AGENT", "CHAIN", "TOKEN", "BLOCK"];
    let word = words[rng.gen_range(0..words.len())];
    let sum: u32 = word
        .chars()
        .map(|c| (c as u32) - ('A' as u32) + 1)
        .sum();
    (
        format!(
            "If A=1, B=2, ..., Z=26, what is the sum of letter values in '{}'?",
            word
        ),
        sum.to_string(),
    )
}
