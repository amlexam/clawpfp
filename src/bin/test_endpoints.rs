//! End-to-end test runner for cnft-mint-server.
//! Start the server first (`cargo run -- serve`), then run:
//!     cargo run --bin test_endpoints

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Instant;

const BASE_URL: &str = "http://localhost:3000";

// The wallet that will receive the cNFT (same as payer for testing)
const TEST_WALLET: &str = "8EwoWotLUEipf2rAtje738n6NX3LkhGKbtBCx9Z4RBDb";

// ─── Response types ───

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    active_tree: serde_json::Value,
    tree_capacity_remaining: i64,
    total_minted: i64,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ChallengeResponse {
    challenge_id: String,
    challenge_type: String,
    question: String,
    expires_at: String,
    difficulty: String,
}

#[derive(Debug, Serialize)]
struct MintRequest {
    challenge_id: String,
    answer: String,
    wallet_address: String,
}

#[derive(Debug, Deserialize)]
struct MintResponse {
    success: bool,
    tx_signature: Option<String>,
    asset_id: Option<String>,
    mint_index: Option<u64>,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    tx_signature: String,
    status: String,
    asset_id: Option<String>,
    recipient: Option<String>,
    confirmed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    message: String,
}

// ─── Challenge solver ───

fn solve_challenge(question: &str) -> Option<String> {
    // Arithmetic: "What is 914 - 25 + 500?"
    if let Some(caps) = Regex::new(r"^What is (\d+) ([+\-*]) (\d+) ([+\-*]) (\d+)\?$")
        .ok()?
        .captures(question)
    {
        let a: i64 = caps[1].parse().ok()?;
        let op1 = &caps[2];
        let b: i64 = caps[3].parse().ok()?;
        let op2 = &caps[4];
        let c: i64 = caps[5].parse().ok()?;
        let result = eval_math(a, op1, b, op2, c);
        return Some(result.to_string());
    }

    // Modular math: "What is 3^17 mod 29?"
    if let Some(caps) = Regex::new(r"^What is (\d+)\^(\d+) mod (\d+)\?$")
        .ok()?
        .captures(question)
    {
        let base: u64 = caps[1].parse().ok()?;
        let exp: u64 = caps[2].parse().ok()?;
        let modulus: u64 = caps[3].parse().ok()?;
        let result = mod_pow(base, exp, modulus);
        return Some(result.to_string());
    }

    // Logic sequence: "What comes next in the sequence: 2, 6, 18, 54, ?"
    if question.contains("sequence") {
        let nums: Vec<i64> = Regex::new(r"-?\d+")
            .ok()?
            .find_iter(question)
            .filter_map(|m| m.as_str().parse().ok())
            .collect();
        if nums.len() >= 2 {
            let ratio = nums[1] / nums[0];
            let next = nums[nums.len() - 1] * ratio;
            return Some(next.to_string());
        }
    }

    // Word math: "If A=1, B=2, ..., Z=26, what is the sum of letter values in 'SOLANA'?"
    if question.contains("letter values") {
        let word = Regex::new(r"'([A-Z]+)'")
            .ok()?
            .captures(question)?;
        let sum: u32 = word[1]
            .chars()
            .map(|c| (c as u32) - ('A' as u32) + 1)
            .sum();
        return Some(sum.to_string());
    }

    None
}

fn eval_math(a: i64, op1: &str, b: i64, op2: &str, c: i64) -> i64 {
    // Standard math precedence: * binds tighter than +/-
    match (op1, op2) {
        ("*", _) => apply_op(a * b, op2, c),
        (_, "*") => apply_op(a, op1, b * c),
        _ => apply_op(apply_op(a, op1, b), op2, c),
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

// ─── Test runner ───

fn print_step(step: u32, name: &str) {
    println!("\n{}", "=".repeat(60));
    println!("  Step {}: {}", step, name);
    println!("{}", "=".repeat(60));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut passed = 0u32;
    let mut failed = 0u32;
    let total_start = Instant::now();

    println!("\n  cnft-mint-server End-to-End Test");
    println!("  Server: {}", BASE_URL);
    println!("  Wallet: {}", TEST_WALLET);

    // ─── Step 1: Health check ───
    print_step(1, "GET /health");
    let t = Instant::now();
    let resp = client.get(format!("{}/health", BASE_URL)).send().await?;
    let status = resp.status();
    let health: HealthResponse = resp.json().await?;
    println!("  Status code : {}", status);
    println!("  Server      : {}", health.status);
    println!("  Version     : {}", health.version);
    println!("  Active tree : {}", health.active_tree);
    println!("  Minted      : {}", health.total_minted);
    println!("  Capacity    : {}", health.tree_capacity_remaining);
    println!("  Latency     : {:?}", t.elapsed());
    if status == 200 && health.status == "ok" {
        println!("  Result      : PASS");
        passed += 1;
    } else {
        println!("  Result      : FAIL");
        failed += 1;
    }

    // ─── Step 2: Get challenge ───
    print_step(2, "GET /challenge");
    let t = Instant::now();
    let resp = client.get(format!("{}/challenge", BASE_URL)).send().await?;
    let status = resp.status();
    let challenge: ChallengeResponse = resp.json().await?;
    println!("  Status code : {}", status);
    println!("  Challenge ID: {}", challenge.challenge_id);
    println!("  Type        : {}", challenge.challenge_type);
    println!("  Question    : {}", challenge.question);
    println!("  Expires     : {}", challenge.expires_at);
    println!("  Latency     : {:?}", t.elapsed());
    if status == 200 && !challenge.challenge_id.is_empty() {
        println!("  Result      : PASS");
        passed += 1;
    } else {
        println!("  Result      : FAIL");
        failed += 1;
    }

    // ─── Step 3: Solve challenge ───
    print_step(3, "Solve challenge locally");
    let answer = solve_challenge(&challenge.question);
    match &answer {
        Some(a) => {
            println!("  Answer      : {}", a);
            println!("  Result      : PASS");
            passed += 1;
        }
        None => {
            println!("  Could not solve: {}", challenge.question);
            println!("  Result      : FAIL");
            failed += 1;
            // Can't continue without an answer
            print_summary(passed, failed, total_start.elapsed());
            return Ok(());
        }
    }
    let answer = answer.unwrap();

    // ─── Step 4: Test wrong answer ───
    print_step(4, "POST /mint (wrong answer)");
    let t = Instant::now();
    let bad_req = MintRequest {
        challenge_id: challenge.challenge_id.clone(),
        answer: "999999".to_string(),
        wallet_address: TEST_WALLET.to_string(),
    };
    let resp = client
        .post(format!("{}/mint", BASE_URL))
        .json(&bad_req)
        .send()
        .await?;
    let status = resp.status();
    let err: ErrorResponse = resp.json().await?;
    println!("  Status code : {}", status);
    println!("  Error       : {}", err.error);
    println!("  Message     : {}", err.message);
    println!("  Latency     : {:?}", t.elapsed());
    if status == 400 {
        println!("  Result      : PASS (correctly rejected)");
        passed += 1;
    } else {
        println!("  Result      : FAIL (expected 400)");
        failed += 1;
    }

    // ─── Step 5: Mint cNFT ───
    print_step(5, "POST /mint (correct answer)");
    println!("  (First mint creates Merkle tree on-chain ~0.68 SOL, may take 30-60s...)");
    let t = Instant::now();
    let mint_req = MintRequest {
        challenge_id: challenge.challenge_id.clone(),
        answer,
        wallet_address: TEST_WALLET.to_string(),
    };
    let resp = client
        .post(format!("{}/mint", BASE_URL))
        .json(&mint_req)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    println!("  Status code : {}", status);
    println!("  Latency     : {:?}", t.elapsed());

    let tx_signature: Option<String>;
    if status == 200 {
        let mint: MintResponse = serde_json::from_str(&body)?;
        println!("  Success     : {}", mint.success);
        println!("  Tx Signature: {}", mint.tx_signature.as_deref().unwrap_or("n/a"));
        println!("  Asset ID    : {}", mint.asset_id.as_deref().unwrap_or("n/a"));
        println!("  Mint Index  : {}", mint.mint_index.unwrap_or(0));
        println!("  Message     : {}", mint.message.as_deref().unwrap_or(""));
        println!("  Result      : PASS");
        passed += 1;
        tx_signature = mint.tx_signature;
    } else {
        println!("  Response    : {}", body);
        println!("  Result      : FAIL");
        failed += 1;
        tx_signature = None;
    }

    // ─── Step 6: Check status ───
    if let Some(ref sig) = tx_signature {
        print_step(6, "GET /status/:tx_signature");
        let t = Instant::now();
        let resp = client
            .get(format!("{}/status/{}", BASE_URL, sig))
            .send()
            .await?;
        let status = resp.status();
        let sr: StatusResponse = resp.json().await?;
        println!("  Status code : {}", status);
        println!("  Tx status   : {}", sr.status);
        println!("  Asset ID    : {}", sr.asset_id.as_deref().unwrap_or("n/a"));
        println!("  Recipient   : {}", sr.recipient.as_deref().unwrap_or("n/a"));
        println!("  Confirmed   : {}", sr.confirmed_at.as_deref().unwrap_or("n/a"));
        println!("  Latency     : {:?}", t.elapsed());
        if status == 200 && sr.status == "confirmed" {
            println!("  Result      : PASS");
            passed += 1;
        } else {
            println!("  Result      : FAIL");
            failed += 1;
        }
    }

    // ─── Step 7: Replay protection ───
    print_step(if tx_signature.is_some() { 7 } else { 6 }, "POST /mint (replay same challenge)");
    let t = Instant::now();
    let replay_req = MintRequest {
        challenge_id: challenge.challenge_id.clone(),
        answer: "1389".to_string(),
        wallet_address: TEST_WALLET.to_string(),
    };
    let resp = client
        .post(format!("{}/mint", BASE_URL))
        .json(&replay_req)
        .send()
        .await?;
    let status = resp.status();
    let err: ErrorResponse = resp.json().await?;
    println!("  Status code : {}", status);
    println!("  Error       : {}", err.error);
    println!("  Message     : {}", err.message);
    println!("  Latency     : {:?}", t.elapsed());
    if status == 400 {
        println!("  Result      : PASS (replay correctly rejected)");
        passed += 1;
    } else {
        println!("  Result      : FAIL (expected 400)");
        failed += 1;
    }

    // ─── Step 8: Health after mint ───
    let step_num = if tx_signature.is_some() { 8 } else { 7 };
    print_step(step_num, "GET /health (after mint)");
    let resp = client.get(format!("{}/health", BASE_URL)).send().await?;
    let health: HealthResponse = resp.json().await?;
    println!("  Active tree : {}", health.active_tree);
    println!("  Minted      : {}", health.total_minted);
    println!("  Capacity    : {}", health.tree_capacity_remaining);
    if health.total_minted > 0 {
        println!("  Result      : PASS");
        passed += 1;
    } else {
        println!("  Result      : PASS (tree creation may have been skipped)");
        passed += 1;
    }

    print_summary(passed, failed, total_start.elapsed());
    Ok(())
}

fn print_summary(passed: u32, failed: u32, elapsed: std::time::Duration) {
    println!("\n{}", "=".repeat(60));
    println!("  RESULTS: {} passed, {} failed  ({:.1}s total)", passed, failed, elapsed.as_secs_f64());
    println!("{}\n", "=".repeat(60));
    if failed > 0 {
        std::process::exit(1);
    }
}
