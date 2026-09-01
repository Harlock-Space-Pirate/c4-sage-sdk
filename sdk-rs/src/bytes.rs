use curve25519_dalek::edwards::CompressedEdwardsY;
use sha2::{Digest, Sha256};

const PDA_MARKER: &[u8] = b"ProgramDerivedAddress";

pub fn encode_pk(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn decode_pk(s: &str) -> Result<[u8; 32], String> {
    let v = bs58::decode(s).into_vec().map_err(|e| e.to_string())?;
    v.try_into().map_err(|_| "pubkey must be 32 bytes".into())
}

fn on_curve(p: &[u8; 32]) -> bool {
    CompressedEdwardsY(*p).decompress().is_some()
}

pub fn find_program_address(seeds: &[&[u8]], program_id: &[u8; 32]) -> Result<(String, u8), String> {
    for bump in (0u8..=255).rev() {
        let mut h = Sha256::new();
        for s in seeds {
            h.update(s);
        }
        h.update([bump]);
        h.update(program_id);
        h.update(PDA_MARKER);
        let hash: [u8; 32] = h.finalize().into();
        if !on_curve(&hash) {
            return Ok((encode_pk(&hash), bump));
        }
    }
    Err("unable to find PDA".into())
}
