//! Read-only Game header. No keys.
//!   cargo run --example headless_read
use c4_sage_sdk::C4SageClient;
use c4_sage_sdk::game::decode_game_header;

fn main() {
    let client = C4SageClient::default_cluster();
    let game = client.config.game.clone();
    match client.fetch_raw(&game) {
        Ok(Some(bytes)) => match decode_game_header(&bytes) {
            Ok(h) => println!(
                "rpc={} bytes={} ceasefire={} mint={}",
                client.config.rpc_url,
                bytes.len(),
                h.global_ceasefire,
                h.atlas_mint
            ),
            Err(e) => eprintln!("decode: {e}"),
        },
        Ok(None) => eprintln!("Game missing on {}", client.config.rpc_url),
        Err(e) => eprintln!("{e}"),
    }
}
