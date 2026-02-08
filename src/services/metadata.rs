use crate::config::Config;
use serde_json::json;

/// Generate the NFT name for a given mint index
pub fn generate_name(config: &Config, mint_index: u64) -> String {
    format!("{} #{}", config.collection_name, mint_index)
}

/// Build the full Metaplex-standard metadata JSON for a cNFT.
pub fn build_metadata_json(
    name: &str,
    symbol: &str,
    description: &str,
    image_url: &str,
    seller_fee_basis_points: u16,
    mint_index: u64,
) -> String {
    let metadata = json!({
        "name": name,
        "symbol": symbol,
        "description": description,
        "image": image_url,
        "external_url": "",
        "attributes": [
            { "trait_type": "Mint Index", "value": mint_index.to_string() },
            { "trait_type": "Mint Method", "value": "Agent Challenge" }
        ],
        "properties": {
            "files": [
                { "uri": image_url, "type": "image/png" }
            ],
            "category": "image"
        },
        "seller_fee_basis_points": seller_fee_basis_points
    });
    serde_json::to_string_pretty(&metadata).unwrap()
}
