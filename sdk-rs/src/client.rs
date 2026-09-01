use crate::catalog::catalog;
use crate::config::{resolve_config, ClusterConfig, ClusterOverrides};
use crate::decode::{decode_account, identify_account};
use crate::fleet::{decode_fleet_readout, FleetReadout};
use crate::ix::{build_ix, ArgVal, BuiltIx};
use crate::rpc;
use crate::tx::{self, KeypairSigner};

/// HTTP is not bundled. Pass account bytes from your RPC layer.
/// Cluster defaults: `chain/registry.json`. Override via `ClusterOverrides` or `C4_*` env.
pub struct C4SageClient {
    pub config: ClusterConfig,
}

impl C4SageClient {
    pub fn new(over: ClusterOverrides) -> Self {
        Self {
            config: resolve_config(over),
        }
    }

    pub fn default_cluster() -> Self {
        Self::new(ClusterOverrides::default())
    }

    pub fn fleet_from_bytes(&self, data: &[u8]) -> Result<FleetReadout, String> {
        let fleet = catalog().account_type("Fleet").ok_or("Fleet missing")?;
        let disc: String = data[..8.min(data.len())].iter().map(|b| format!("{b:02x}")).collect();
        if disc != fleet.discriminator {
            return Err("not a Fleet account".into());
        }
        decode_fleet_readout(data)
    }

    pub fn build_ix(
        &self,
        name: &str,
        accounts: &[(&str, &str)],
        args: &[(&str, ArgVal)],
    ) -> Result<BuiltIx, String> {
        build_ix(name, accounts, args, Some(&self.config.program_id))
    }

    pub fn identify(&self, data: &[u8]) -> Option<&'static crate::catalog::AccountTypeDef> {
        identify_account(data, &catalog().account_types)
    }

    pub fn decode(&self, data: &[u8]) -> Option<Vec<crate::decode::DecodedField>> {
        let t = self.identify(data)?;
        Some(decode_account(data, t))
    }

    pub fn fetch_raw(&self, pubkey: &str) -> Result<Option<Vec<u8>>, String> {
        rpc::get_account_data(&self.config.rpc_url, pubkey)
    }

    pub fn simulate(&self, ixs: &[BuiltIx], fee_payer: &str) -> Result<serde_json::Value, String> {
        tx::simulate(&self.config.rpc_url, ixs, fee_payer)
    }

    pub fn sign_and_send(&self, ixs: &[BuiltIx], signer: &KeypairSigner) -> Result<String, String> {
        tx::sign_and_send(&self.config.rpc_url, ixs, signer)
    }
}
