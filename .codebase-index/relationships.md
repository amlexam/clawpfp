## Trait Implementations

From < & Challenge >  <- ChallengeResponse
IntoResponse          <- AppError
TryFrom < TreeRow >   <- TreeInfo
std :: fmt :: Display <- ChallengeType

## Error Chains

& Challenge -> ChallengeResponse

## Module Dependencies

config                 -> (no internal deps)
crate                  -> (no internal deps)
db                     -> (no internal deps)
db::challenges         -> models::challenge
db::mints              -> (no internal deps)
db::trees              -> models::tree
error                  -> (no internal deps)
models                 -> (no internal deps)
models::challenge      -> (no internal deps)
models::mint           -> (no internal deps)
models::tree           -> (no internal deps)
routes                 -> state
routes::challenge      -> db, error, models::challenge, services, state
routes::health         -> db, state
routes::mint           -> db, error, models::mint, services, state
routes::status         -> db, error, models::mint, state
services               -> (no internal deps)
services::bubblegum    -> (no internal deps)
services::challenge    -> models::challenge
services::irys         -> (no internal deps)
services::metadata     -> config
services::solana       -> (no internal deps)
services::tree_manager -> config, db, models::tree, services
setup                  -> (no internal deps)
state                  -> config, services::tree_manager

## Key Types (referenced from 3+ modules)

AppState          — used in 6 modules
Error             — used in 5 modules
PgPool            — used in 5 modules
Pubkey            — used in 5 modules
AppError          — used in 4 modules
Config            — used in 4 modules
Json              — used in 4 modules
Keypair           — used in 4 modules
RpcClient         — used in 4 modules
State             — used in 4 modules
Challenge         — used in 3 modules
ChallengeResponse — used in 3 modules
MintRequest       — used in 3 modules
MintResponse      — used in 3 modules
StatusResponse    — used in 3 modules

