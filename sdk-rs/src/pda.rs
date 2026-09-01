use crate::bytes::{decode_pk, find_program_address};
use crate::catalog::{catalog, PdaDef};

pub fn find_pda(name: &str, seeds_named: &[(&str, &str)]) -> Result<(String, u8), String> {
    let c = catalog();
    let def = c.pda(name).ok_or_else(|| format!("unknown PDA {name}"))?;
    if def.confidence != "certain" {
        return Err(format!("PDA {name} is not certain"));
    }
    let pid = decode_pk(def.program_id.as_deref().unwrap_or(&c.pda_program_id))?;
    let seed_bufs = seed_bytes(def, seeds_named)?;
    let refs: Vec<&[u8]> = seed_bufs.iter().map(|b| b.as_slice()).collect();
    find_program_address(&refs, &pid)
}

fn lookup<'a>(named: &[(&str, &'a str)], keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some((_, v)) = named.iter().find(|(n, _)| n == k) {
            return Some(*v);
        }
    }
    None
}

fn seed_bytes(def: &PdaDef, named: &[(&str, &str)]) -> Result<Vec<Vec<u8>>, String> {
    let mut out = Vec::new();
    for s in &def.seeds {
        match s.kind.as_str() {
            "literal" => out.push(s.value.clone().unwrap_or_default().into_bytes()),
            "literal_bytes" => {
                let v = s.value.clone().unwrap_or_default();
                if let Some((_, n)) = v.split_once('×') {
                    let n: usize = n.parse().unwrap_or(32);
                    out.push(vec![0u8; n]);
                } else if let Some(hex) = v.strip_prefix("0x") {
                    out.push(hex::decode_like(hex)?);
                }
            }
            "account" | "field" => {
                let ty = s.ty.as_deref().unwrap_or("");
                let role = s.role.as_deref().unwrap_or("");
                let key = role.split('.').next().unwrap_or(role);
                let v = lookup(named, &[ty, key, role])
                    .ok_or_else(|| format!("missing seed {ty} ({role})"))?;
                if ty == "[u8;32]" || role.contains("label") {
                    let mut b = vec![0u8; 32];
                    let raw = v.as_bytes();
                    let n = raw.len().min(32);
                    b[..n].copy_from_slice(&raw[..n]);
                    out.push(b);
                } else {
                    out.push(decode_pk(v)?.to_vec());
                }
            }
            k => return Err(format!("unknown seed kind {k}")),
        }
    }
    Ok(out)
}

mod hex {
    pub fn decode_like(s: &str) -> Result<Vec<u8>, String> {
        let s: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

pub fn starbase_player_pda(star_system: &str, character: &str) -> Result<(String, u8), String> {
    find_pda(
        "StarbasePlayer",
        &[("StarSystem", star_system), ("Character", character)],
    )
}

pub fn character_pda(profile: &str, game: &str) -> Result<(String, u8), String> {
    find_pda("Character", &[("Profile", profile), ("Game", game)])
}

pub fn fleet_pda(game: &str, profile: &str, label: &str) -> Result<(String, u8), String> {
    find_pda(
        "Fleet",
        &[
            ("Game", game),
            ("Profile", profile),
            ("[u8;32]", label),
            ("fleet.label32", label),
        ],
    )
}
