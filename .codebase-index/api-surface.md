# Crate: clawpfp (bin)

# crate
<!-- file: src/main.rs -->

## Functions

async fn main() -> anyhow :: Result < () >;


---

# crate::config
<!-- file: src/config.rs -->

## Types

pub struct Config {
    pub solana_rpc_url: String,
    pub payer_keypair_json: String,
    pub merkle_tree_max_depth: u32,
    pub merkle_tree_max_buffer_size: u32,
    pub merkle_tree_canopy_depth: u32,
    pub collection_mint: Option < Pubkey >,
    pub collection_name: String,
    pub collection_symbol: String,
    pub base_metadata_uri: String,
    pub seller_fee_basis_points: u16,
    pub collection_description: String,
    pub collection_image_url: String,
    pub irys_node_url: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
    pub challenge_expiry_seconds: i64,
}


## Impl Config

impl Config {
    pub fn from_env() -> anyhow :: Result < Self >;
}


---

# crate::db
<!-- file: src/db/mod.rs -->

---

# crate::db::challenges
<!-- file: src/db/challenges.rs -->

## Functions

pub async fn insert_challenge(pool : & PgPool, challenge : & Challenge) -> Result < () , sqlx :: Error >;

pub async fn get_challenge(pool : & PgPool, id : & str) -> Result < Option < Challenge > , sqlx :: Error >;

pub async fn mark_challenge_consumed(pool : & PgPool, id : & str) -> Result < () , sqlx :: Error >;

pub async fn expire_challenge(pool : & PgPool, id : & str) -> Result < () , sqlx :: Error >;


---

# crate::db::mints
<!-- file: src/db/mints.rs -->

## Functions

pub async fn insert_mint(pool : & PgPool, asset_id : & str, tree_address : & str, leaf_index : u64, recipient_wallet : & str, metadata_uri : & str, metadata_name : & str, tx_signature : & str, challenge_id : & str) -> Result < () , sqlx :: Error >;

pub async fn get_mint_by_tx(pool : & PgPool, tx_signature : & str) -> Result < Option < (String , String , String , String , String) > , sqlx :: Error >;

pub async fn get_total_minted(pool : & PgPool) -> Result < i64 , sqlx :: Error >;


---

# crate::db::trees
<!-- file: src/db/trees.rs -->

## Functions

pub async fn get_active_tree(pool : & PgPool) -> Result < Option < TreeRow > , sqlx :: Error >;

pub async fn insert_tree(pool : & PgPool, address : & str, max_depth : u32, max_buffer_size : u32, canopy_depth : u32, max_capacity : u64, collection_mint : Option < & str >, creation_tx : Option < & str >) -> Result < () , sqlx :: Error >;

pub async fn deactivate_tree(pool : & PgPool, address : & str) -> Result < () , sqlx :: Error >;

pub async fn increment_tree_leaf_index(pool : & PgPool, address : & str) -> Result < () , sqlx :: Error >;

pub async fn get_tree_capacity_remaining(pool : & PgPool) -> Result < i64 , sqlx :: Error >;


---

# crate::error
<!-- file: src/error.rs -->

## Types

pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Gone(String),
    Internal(String),
    Anyhow(anyhow :: Error),
    Sqlx(sqlx :: Error),
}


## Impl IntoResponse for AppError

impl IntoResponse for AppError {
    fn into_response(self) -> Response;
}


---

# crate::models
<!-- file: src/models/mod.rs -->

---

# crate::models::challenge
<!-- file: src/models/challenge.rs -->

## Types

pub enum ChallengeType {
    Arithmetic,
    ModularMath,
    LogicSequence,
    WordMath,
}

pub struct Challenge {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub question: String,
    pub answer: String,
    pub expires_at: chrono :: DateTime < chrono :: Utc >,
    pub status: String,
}

/// Response returned to the client for GET /challenge
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub challenge_type: String,
    pub question: String,
    pub expires_at: chrono :: DateTime < chrono :: Utc >,
    pub difficulty: String,
}


## Impl std :: fmt :: Display for ChallengeType

impl std :: fmt :: Display for ChallengeType {
    fn fmt(& self, f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result;
}


## Impl ChallengeType

impl ChallengeType {
    pub fn from_str_loose(s : & str) -> Self;
}


## Impl From < & Challenge > for ChallengeResponse

impl From < & Challenge > for ChallengeResponse {
    fn from(c : & Challenge) -> Self;
}


---

# crate::models::mint
<!-- file: src/models/mint.rs -->

## Types

pub struct MintRequest {
    pub challenge_id: String,
    pub answer: String,
    pub wallet_address: String,
}

pub struct MintResponse {
    pub success: bool,
    pub tx_signature: String,
    pub asset_id: String,
    pub mint_index: u64,
    pub message: String,
}

pub struct StatusResponse {
    pub tx_signature: String,
    pub status: String,
    pub asset_id: Option < String >,
    pub recipient: Option < String >,
    pub confirmed_at: Option < String >,
}


---

# crate::models::tree
<!-- file: src/models/tree.rs -->

## Types

pub struct TreeInfo {
    pub address: Pubkey,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
    pub max_capacity: u64,
    pub current_leaf_index: u64,
    pub is_active: bool,
}

/// Database row representation for merkle_trees table
pub struct TreeRow {
    pub id: i64,
    pub address: String,
    pub max_depth: i32,
    pub max_buffer_size: i32,
    pub canopy_depth: i32,
    pub max_capacity: i64,
    pub current_leaf_index: i64,
    pub is_active: bool,
}


## Impl TryFrom < TreeRow > for TreeInfo

impl TryFrom < TreeRow > for TreeInfo {
    type Error = anyhow :: Error;
    fn try_from(row : TreeRow) -> Result < Self , Self :: Error >;
}


---

# crate::routes
<!-- file: src/routes/mod.rs -->

## Functions

async fn serve_skill_md() -> Response;

pub fn create_router(state : Arc < AppState >) -> Router;


---

# crate::routes::health
<!-- file: src/routes/health.rs -->

## Functions

pub async fn health_handler(State (state) : State < Arc < AppState > >) -> Json < Value >;


---

# crate::routes::challenge
<!-- file: src/routes/challenge.rs -->

## Functions

pub async fn challenge_handler(State (state) : State < Arc < AppState > >) -> Result < Json < ChallengeResponse > , AppError >;


---

# crate::routes::mint
<!-- file: src/routes/mint.rs -->

## Functions

pub async fn mint_handler(State (state) : State < Arc < AppState > >, Json (req) : Json < MintRequest >) -> Result < Json < MintResponse > , AppError >;


---

# crate::routes::status
<!-- file: src/routes/status.rs -->

## Functions

pub async fn status_handler(State (state) : State < Arc < AppState > >, Path (tx_signature) : Path < String >) -> Result < Json < StatusResponse > , AppError >;


---

# crate::services
<!-- file: src/services/mod.rs -->

---

# crate::services::challenge
<!-- file: src/services/challenge.rs -->

## Functions

pub fn generate_challenge(expiry_seconds : i64) -> Challenge;

pub fn verify_challenge_answer(challenge : & Challenge, submitted : & str) -> bool;

fn generate_arithmetic(rng : & mut impl Rng) -> (String , String);

fn eval_expression(a : i64, op1 : & str, b : i64, op2 : & str, c : i64) -> i64;

fn apply_op(a : i64, op : & str, b : i64) -> i64;

fn generate_modular_math(rng : & mut impl Rng) -> (String , String);

fn mod_pow(mut base : u64, mut exp : u64, modulus : u64) -> u64;

fn generate_logic_sequence(rng : & mut impl Rng) -> (String , String);

fn generate_word_math(rng : & mut impl Rng) -> (String , String);


---

# crate::services::bubblegum
<!-- file: src/services/bubblegum.rs -->

## Functions

pub fn bubblegum_program_id() -> Pubkey;

pub fn spl_account_compression_id() -> Pubkey;

pub fn spl_noop_id() -> Pubkey;

pub fn token_metadata_program_id() -> Pubkey;

pub fn build_mint_to_collection_ix(payer : & Pubkey, merkle_tree : & Pubkey, leaf_owner : & Pubkey, collection_mint : & Pubkey, name : String, symbol : String, uri : String, seller_fee_basis_points : u16) -> solana_sdk :: instruction :: Instruction;

/// Derive the asset ID for a cNFT given the tree address and leaf index
pub fn derive_asset_id(merkle_tree : & Pubkey, leaf_index : u64) -> Pubkey;

/// Calculate the required account size for a concurrent Merkle tree.
/// Matches the exact layout of SPL Account Compression's ConcurrentMerkleTree.
pub fn get_merkle_tree_size(max_depth : u32, max_buffer_size : u32, canopy_depth : u32) -> usize;


## Constants

pub const BUBBLEGUM_PROGRAM_ID: & str;

pub const SPL_ACCOUNT_COMPRESSION_ID: & str;

pub const SPL_NOOP_ID: & str;

pub const TOKEN_METADATA_PROGRAM_ID: & str;


---

# crate::services::tree_manager
<!-- file: src/services/tree_manager.rs -->

## Types

pub struct TreeManager {
    pub db: PgPool,
    pub rpc_client: Arc < RpcClient >,
    pub payer: Arc < Keypair >,
    pub config: Config,
}


## Impl TreeManager

impl TreeManager {
    pub fn new(db : PgPool, rpc_client : Arc < RpcClient >, payer : Arc < Keypair >, config : Config) -> Self;
    pub async fn get_active_tree(& self) -> anyhow :: Result < TreeInfo >;
    pub async fn create_and_register_tree(& self) -> anyhow :: Result < TreeInfo >;
    async fn create_merkle_tree(& self, max_depth : u32, max_buffer_size : u32, canopy_depth : u32) -> anyhow :: Result < (Pubkey , String) >;
}


---

# crate::services::solana
<!-- file: src/services/solana.rs -->

## Functions

/// Check the confirmation status of a transaction
pub async fn get_transaction_status(rpc_client : & RpcClient, tx_signature : & str) -> anyhow :: Result < Option < String > >;


---

# crate::services::metadata
<!-- file: src/services/metadata.rs -->

## Functions

/// Generate the NFT name for a given mint index
pub fn generate_name(config : & Config, mint_index : u64) -> String;

/// Build the full Metaplex-standard metadata JSON for a cNFT.
/// The image_url may contain `{mint_index}` which will be replaced with the actual mint index,
/// allowing each NFT to have a unique generated image (e.g. DiceBear avatars).
pub fn build_metadata_json(name : & str, symbol : & str, description : & str, image_url : & str, seller_fee_basis_points : u16, mint_index : u64) -> String;


---

# crate::services::irys
<!-- file: src/services/irys.rs -->

## Functions

fn deep_hash_blob(data : & [u8]) -> Vec < u8 >;

fn deep_hash_list(items : & [& [u8]]) -> Vec < u8 >;

/// Zigzag-encode a signed i64 into an unsigned u64 (Avro "long" wire format).
fn avro_zigzag(n : i64) -> u64;

/// Variable-length encode a u64 into Avro varint bytes.
fn avro_varint(mut n : u64) -> Vec < u8 >;

/// Encode an i64 as an Avro "long" (zigzag + varint).
fn avro_long(n : i64) -> Vec < u8 >;

fn serialize_tags(tags : & [(& str , & str)]) -> Vec < u8 >;

fn create_signed_data_item(data : & [u8], tags : & [(& str , & str)], keypair : & Keypair) -> anyhow :: Result < Vec < u8 > >;

/// Upload data to Irys and return the full gateway URL.
/// 
/// `content_type` should be e.g. "application/json" or "image/png".
/// Returns a URL like `https://devnet.irys.xyz/{tx_id}`.
pub async fn upload(http_client : & reqwest :: Client, data : & [u8], content_type : & str, keypair : & Keypair, node_url : & str) -> anyhow :: Result < String >;


---

# crate::setup
<!-- file: src/setup.rs -->

## Types

struct CreateMetadataAccountArgsV3 {
    data: DataV2,
    is_mutable: bool,
    collection_details: Option < CollectionDetails >,
}

struct DataV2 {
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    creators: Option < Vec < CreatorData > >,
    collection: Option < CollectionData >,
    uses: Option < UsesData >,
}

struct CreatorData {
    address: [u8 ; 32],
    verified: bool,
    share: u8,
}

struct CollectionData {
    verified: bool,
    key: [u8 ; 32],
}

struct UsesData {
    use_method: u8,
    remaining: u64,
    total: u64,
}

enum CollectionDetails {
    V1 { size: u64 },
}

struct CreateMasterEditionArgs {
    max_supply: Option < u64 >,
}


## Functions

fn token_program_id() -> Pubkey;

fn associated_token_program_id() -> Pubkey;

fn token_metadata_program_id() -> Pubkey;

fn rent_sysvar_id() -> Pubkey;

fn initialize_mint2_ix(mint : & Pubkey, mint_authority : & Pubkey, freeze_authority : Option < & Pubkey >, decimals : u8) -> Instruction;

fn mint_to_ix(mint : & Pubkey, destination : & Pubkey, authority : & Pubkey, amount : u64) -> Instruction;

fn derive_ata(wallet : & Pubkey, mint : & Pubkey) -> Pubkey;

fn create_ata_idempotent_ix(funder : & Pubkey, wallet : & Pubkey, mint : & Pubkey) -> Instruction;

fn derive_metadata_pda(mint : & Pubkey) -> Pubkey;

fn derive_master_edition_pda(mint : & Pubkey) -> Pubkey;

fn create_metadata_account_v3_ix(metadata : & Pubkey, mint : & Pubkey, mint_authority : & Pubkey, payer : & Pubkey, update_authority : & Pubkey, args : CreateMetadataAccountArgsV3) -> Instruction;

fn create_master_edition_v3_ix(edition : & Pubkey, mint : & Pubkey, update_authority : & Pubkey, mint_authority : & Pubkey, payer : & Pubkey, metadata : & Pubkey) -> Instruction;

pub async fn setup_collection(rpc_client : & RpcClient, payer : & Keypair, name : & str, symbol : & str, uri : & str) -> anyhow :: Result < Pubkey >;


## Constants

const TOKEN_PROGRAM_ID: & str;

const ASSOCIATED_TOKEN_PROGRAM_ID: & str;

const TOKEN_METADATA_PROGRAM_ID: & str;

const RENT_SYSVAR_ID: & str;

const SPL_TOKEN_MINT_SIZE: u64;


---

# crate::state
<!-- file: src/state.rs -->

## Types

pub struct AppState {
    pub config: Config,
    pub rpc_client: Arc < RpcClient >,
    pub payer: Arc < Keypair >,
    pub db: PgPool,
    pub tree_manager: TreeManager,
    pub http_client: reqwest :: Client,
}


---

# Crate: test_endpoints (bin)

# crate
<!-- file: src/bin/test_endpoints.rs -->

## Types

struct HealthResponse {
    status: String,
    active_tree: serde_json :: Value,
    tree_capacity_remaining: i64,
    total_minted: i64,
    version: String,
}

struct ChallengeResponse {
    challenge_id: String,
    challenge_type: String,
    question: String,
    expires_at: String,
    difficulty: String,
}

struct MintRequest {
    challenge_id: String,
    answer: String,
    wallet_address: String,
}

struct MintResponse {
    success: bool,
    tx_signature: Option < String >,
    asset_id: Option < String >,
    mint_index: Option < u64 >,
    message: Option < String >,
    error: Option < String >,
}

struct StatusResponse {
    tx_signature: String,
    status: String,
    asset_id: Option < String >,
    recipient: Option < String >,
    confirmed_at: Option < String >,
}

struct ErrorResponse {
    success: bool,
    error: String,
    message: String,
}


## Functions

fn base_url() -> String;

fn solve_challenge(question : & str) -> Option < String >;

fn eval_math(a : i64, op1 : & str, b : i64, op2 : & str, c : i64) -> i64;

fn apply_op(a : i64, op : & str, b : i64) -> i64;

fn mod_pow(mut base : u64, mut exp : u64, modulus : u64) -> u64;

fn print_step(step : u32, name : & str);

async fn main() -> anyhow :: Result < () >;

fn print_summary(passed : u32, failed : u32, elapsed : std :: time :: Duration);


## Constants

const TEST_WALLET: & str;


---

