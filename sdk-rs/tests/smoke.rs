use c4_sage_sdk::catalog::catalog;
use c4_sage_sdk::pda::starbase_player_pda;
use sha2::{Digest, Sha256};

fn disc(prefix: &str, name: &str) -> String {
    let h = Sha256::digest(format!("{prefix}{name}").as_bytes());
    h[..8].iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn instruction_discs() {
    let c = catalog();
    let w = c.instruction("withdraw_atlas").expect("withdraw_atlas");
    assert_eq!(w.discriminator, disc("global:", "withdraw_atlas"));
}

#[test]
fn account_discs() {
    let c = catalog();
    assert_eq!(c.account_type("Fleet").unwrap().discriminator, disc("account:", "Fleet"));
}

#[test]
fn pda_deterministic() {
    let g = &catalog().game;
    let a = starbase_player_pda(g, g).unwrap();
    let b = starbase_player_pda(g, g).unwrap();
    assert_eq!(a, b);
}

#[test]
fn registry_is_single_cluster_source() {
    let c = catalog();
    assert!(!c.rpc.is_empty());
    assert!(!c.program_id.is_empty());
    assert!(!c.game.is_empty());
}

#[test]
fn config_override() {
    let cfg = c4_sage_sdk::resolve_config(c4_sage_sdk::ClusterOverrides {
        rpc_url: Some("http://127.0.0.1:8899".into()),
        ..Default::default()
    });
    assert_eq!(cfg.rpc_url, "http://127.0.0.1:8899");
}
