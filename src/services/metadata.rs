use crate::config::Config;
use serde_json::json;

/// Generate the NFT name for a given mint index
pub fn generate_name(config: &Config, mint_index: u64) -> String {
    format!("{} #{}", config.collection_name, mint_index)
}

/// Build the full Metaplex-standard metadata JSON for a cNFT.
/// The image_url may contain `{mint_index}` which will be replaced with the actual mint index,
/// allowing each NFT to have a unique generated image (e.g. DiceBear avatars).
pub fn build_metadata_json(
    name: &str,
    symbol: &str,
    description: &str,
    image_url: &str,
    seller_fee_basis_points: u16,
    mint_index: u64,
) -> String {
    let image = image_url.replace("{mint_index}", &mint_index.to_string());
    let file_type = if image.contains("/svg") { "image/svg+xml" } else { "image/png" };
    let metadata = json!({
        "name": name,
        "symbol": symbol,
        "description": description,
        "image": image,
        "external_url": "",
        "attributes": [
            { "trait_type": "Mint Index", "value": mint_index.to_string() },
            { "trait_type": "Mint Method", "value": "Agent Challenge" }
        ],
        "properties": {
            "files": [
                { "uri": image, "type": file_type }
            ],
            "category": "image"
        },
        "seller_fee_basis_points": seller_fee_basis_points
    });
    serde_json::to_string_pretty(&metadata).unwrap()
}
