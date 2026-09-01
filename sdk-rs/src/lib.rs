//! C4 SAGE (Zink) SDK. Catalog: `chain/`. Cluster defaults: `chain/registry.json`.
//! Override via `ClusterOverrides` or env `C4_RPC` / `C4_PROGRAM_ID` / `C4_GAME`.

pub mod bytes;
pub mod catalog;
pub mod client;
pub mod config;
pub mod decode;
pub mod fleet;
pub mod game;
pub mod ix;
pub mod pda;
pub mod rpc;
pub mod tx;

pub use catalog::{catalog, Catalog};
pub use client::C4SageClient;
pub use config::{resolve_config, ClusterConfig, ClusterOverrides};
pub use fleet::{decode_fleet_readout, FleetReadout};
pub use ix::{build_ix, BuiltIx};
pub use pda::{character_pda, fleet_pda, starbase_player_pda};
pub use tx::{compile_message, sign_and_send, simulate, KeypairSigner};
