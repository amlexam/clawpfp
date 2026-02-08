//! Upload data to Arweave via Irys using ANS-104 signed data items.

use sha2::{Digest, Sha384};
use solana_sdk::signer::{keypair::Keypair, Signer};

// ─── ANS-104 Deep Hash ───

fn deep_hash_blob(data: &[u8]) -> Vec<u8> {
    let tag_input = [b"blob".as_slice(), data.len().to_string().as_bytes()].concat();
    let tag_hash = Sha384::digest(&tag_input);
    let data_hash = Sha384::digest(data);
    let combined = [tag_hash.as_slice(), data_hash.as_slice()].concat();
    Sha384::digest(&combined).to_vec()
}

fn deep_hash_list(items: &[&[u8]]) -> Vec<u8> {
    let tag_input = [b"list".as_slice(), items.len().to_string().as_bytes()].concat();
    let mut acc = Sha384::digest(&tag_input).to_vec();
    for item in items {
        let item_hash = deep_hash_blob(item);
        let combined = [acc.as_slice(), item_hash.as_slice()].concat();
        acc = Sha384::digest(&combined).to_vec();
    }
    acc
}

// ─── Avro binary encoding helpers ───

/// Zigzag-encode a signed i64 into an unsigned u64 (Avro "long" wire format).
fn avro_zigzag(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Variable-length encode a u64 into Avro varint bytes.
fn avro_varint(mut n: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        if n <= 0x7F {
            buf.push(n as u8);
            break;
        }
        buf.push(((n & 0x7F) | 0x80) as u8);
        n >>= 7;
    }
    buf
}

/// Encode an i64 as an Avro "long" (zigzag + varint).
fn avro_long(n: i64) -> Vec<u8> {
    avro_varint(avro_zigzag(n))
}

// ─── Tag serialization (Avro binary encoding per ANS-104) ───

fn serialize_tags(tags: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::new();
    if tags.is_empty() {
        buf.push(0x00); // empty array terminator
        return buf;
    }
    // Avro array: block count, then items, then 0 terminator
    buf.extend_from_slice(&avro_long(tags.len() as i64));
    for (name, value) in tags {
        let name_bytes = name.as_bytes();
        let value_bytes = value.as_bytes();
        // Avro bytes: length as avro long, then raw bytes
        buf.extend_from_slice(&avro_long(name_bytes.len() as i64));
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&avro_long(value_bytes.len() as i64));
        buf.extend_from_slice(value_bytes);
    }
    buf.push(0x00); // array terminator
    buf
}

// ─── ANS-104 Data Item ───

fn create_signed_data_item(
    data: &[u8],
    tags: &[(&str, &str)],
    keypair: &Keypair,
) -> anyhow::Result<Vec<u8>> {
    let owner = keypair.pubkey().to_bytes();
    let tags_bytes = serialize_tags(tags);

    // Deep hash the data item fields
    let sig_type_str = b"2"; // ED25519
    let items: Vec<&[u8]> = vec![
        b"dataitem",
        b"1",
        sig_type_str,
        &owner,
        b"", // no target
        b"", // no anchor
        &tags_bytes,
        data,
    ];
    let hash = deep_hash_list(&items);

    // Sign the deep hash with ed25519
    let signature = keypair.sign_message(&hash);

    // Build binary data item
    let sig_type: u16 = 2; // ED25519
    let mut item = Vec::new();
    item.extend_from_slice(&sig_type.to_le_bytes());   // 2 bytes
    item.extend_from_slice(signature.as_ref());         // 64 bytes
    item.extend_from_slice(&owner);                     // 32 bytes
    item.push(0); // target not present
    item.push(0); // anchor not present
    item.extend_from_slice(&(tags.len() as u64).to_le_bytes());       // 8 bytes
    item.extend_from_slice(&(tags_bytes.len() as u64).to_le_bytes()); // 8 bytes
    item.extend_from_slice(&tags_bytes);
    item.extend_from_slice(data);

    Ok(item)
}

// ─── Public API ───

/// Upload data to Irys and return the full gateway URL.
///
/// `content_type` should be e.g. "application/json" or "image/png".
/// Returns a URL like `https://devnet.irys.xyz/{tx_id}`.
pub async fn upload(
    http_client: &reqwest::Client,
    data: &[u8],
    content_type: &str,
    keypair: &Keypair,
    node_url: &str,
) -> anyhow::Result<String> {
    let tags = vec![("Content-Type", content_type)];
    let data_item = create_signed_data_item(data, &tags, keypair)?;

    let url = format!("{}/tx/solana", node_url);
    tracing::info!("Uploading {} bytes to Irys ({})...", data.len(), url);

    let resp = http_client
        .post(&url)
        .header("Content-Type", "application/octet-stream")
        .body(data_item)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 402 {
            anyhow::bail!(
                "Irys account has insufficient funds. \
                 Fund it with: `irys fund 5000000 -n devnet -t solana -w <keypair>` \
                 (5000000 lamports = 0.005 SOL). Raw error: {}",
                body
            );
        }
        anyhow::bail!("Irys upload failed (HTTP {}): {}", status, body);
    }

    let json: serde_json::Value = resp.json().await?;
    let tx_id = json["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'id' in Irys response: {:?}", json))?;

    let gateway_url = format!("{}/{}", node_url, tx_id);
    tracing::info!("Uploaded to Irys: {}", gateway_url);

    Ok(gateway_url)
}
