use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::str::FromStr;

/// Check the confirmation status of a transaction
pub async fn get_transaction_status(
    rpc_client: &RpcClient,
    tx_signature: &str,
) -> anyhow::Result<Option<String>> {
    let sig = Signature::from_str(tx_signature)?;
    let status = rpc_client.get_signature_statuses(&[sig]).await?;

    Ok(status.value[0].as_ref().map(|s| {
        if s.err.is_some() {
            "failed".to_string()
        } else if s.confirmations.is_none() || s.confirmations.unwrap() > 0 {
            "confirmed".to_string()
        } else {
            "processing".to_string()
        }
    }))
}
