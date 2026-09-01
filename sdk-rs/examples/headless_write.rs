//! YOU sign. Set C4_SECRET_KEY_JSON to a JSON array of 32/64 bytes (lancer key only).
//! Default simulates. C4_SEND=1 submits.
use c4_sage_sdk::ix::{build_ix, ArgVal};
use c4_sage_sdk::tx::KeypairSigner;
use c4_sage_sdk::C4SageClient;

fn main() {
    let raw = std::env::var("C4_SECRET_KEY_JSON").expect("C4_SECRET_KEY_JSON=[u8,…] lancer key only");
    let nums: Vec<u8> = serde_json::from_str(&raw).expect("json array of bytes");
    let signer = KeypairSigner::from_secret(&nums).expect("keypair");
    let client = C4SageClient::default_cluster();
    let pk = signer.public_key();
    let profile = std::env::var("C4_PROFILE").unwrap_or_else(|_| pk.clone());
    let character = std::env::var("C4_CHARACTER").unwrap_or_else(|_| pk.clone());
    let ix = build_ix(
        "withdraw_atlas",
        &[
            ("profileValidationSigner", pk.as_str()),
            ("profileValidationProfile", profile.as_str()),
            ("profileValidationProgram", client.config.profile_program.as_str()),
            ("character", character.as_str()),
            ("game", client.config.game.as_str()),
            ("gameAtlasVault", client.config.game.as_str()),
            ("destination", client.config.game.as_str()),
            ("currencyCache", client.config.game.as_str()),
            ("tokenProgram", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        ],
        &[("keyIndex", ArgVal::U64(0)), ("amount", ArgVal::U64(1))],
        Some(&client.config.program_id),
    )
    .expect("ix");
    let sim = client.simulate(&[ix.clone()], &pk).expect("sim");
    println!("sim {}", sim);
    if std::env::var("C4_SEND").ok().as_deref() == Some("1") {
        println!("sent {}", client.sign_and_send(&[ix], &signer).expect("send"));
    }
}
