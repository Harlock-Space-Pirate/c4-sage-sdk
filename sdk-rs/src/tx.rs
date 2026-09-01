//! Legacy Solana transaction compile / sign / send. The crate never stores keys.

use crate::bytes::{decode_pk, encode_pk};
use crate::ix::BuiltIx;
use crate::rpc;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer as EdSigner};
use serde_json::Value;

pub struct KeypairSigner {
    kp: Keypair,
}

impl KeypairSigner {
    /// 32-byte seed or 64-byte Solana secret. Headless / lancer wallets only.
    pub fn from_secret(secret: &[u8]) -> Result<Self, String> {
        let kp = if secret.len() == 64 {
            Keypair::from_bytes(secret).map_err(|e| e.to_string())?
        } else if secret.len() == 32 {
            let sk = SecretKey::from_bytes(secret).map_err(|e| e.to_string())?;
            let pk = PublicKey::from(&sk);
            Keypair {
                secret: sk,
                public: pk,
            }
        } else {
            return Err("secret key must be 32 or 64 bytes".into());
        };
        Ok(Self { kp })
    }

    pub fn public_key(&self) -> String {
        encode_pk(self.kp.public.as_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.kp.sign(message).to_bytes()
    }
}

fn compact_u16(n: usize) -> Vec<u8> {
    if n < 0x80 {
        vec![n as u8]
    } else if n < 0x4000 {
        vec![((n as u8) & 0x7f) | 0x80, (n >> 7) as u8]
    } else {
        vec![
            ((n as u8) & 0x7f) | 0x80,
            (((n >> 7) as u8) & 0x7f) | 0x80,
            (n >> 14) as u8,
        ]
    }
}

#[derive(Debug, Clone)]
struct Meta {
    pubkey: String,
    signer: bool,
    writable: bool,
}

#[derive(Debug, Clone)]
pub struct CompiledMessage {
    pub bytes: Vec<u8>,
    pub account_keys: Vec<String>,
    pub num_required_signatures: usize,
}

pub fn compile_message(
    ixs: &[BuiltIx],
    fee_payer: &str,
    recent_blockhash: &str,
) -> Result<CompiledMessage, String> {
    let mut map: Vec<Meta> = Vec::new();
    let bump = |map: &mut Vec<Meta>, pk: &str, signer: bool, writable: bool| {
        if let Some(m) = map.iter_mut().find(|m| m.pubkey == pk) {
            m.signer |= signer;
            m.writable |= writable;
        } else {
            map.push(Meta {
                pubkey: pk.to_string(),
                signer,
                writable,
            });
        }
    };
    bump(&mut map, fee_payer, true, true);
    for ix in ixs {
        bump(&mut map, &ix.program_id, false, false);
        for k in &ix.keys {
            bump(&mut map, &k.pubkey, k.is_signer, k.is_writable);
        }
    }
    let mut signed_w: Vec<Meta> = map.iter().filter(|m| m.signer && m.writable).cloned().collect();
    signed_w.sort_by_key(|m| if m.pubkey == fee_payer { 0u8 } else { 1 });
    let signed_r: Vec<Meta> = map.iter().filter(|m| m.signer && !m.writable).cloned().collect();
    let unsigned_w: Vec<Meta> = map.iter().filter(|m| !m.signer && m.writable).cloned().collect();
    let unsigned_r: Vec<Meta> = map.iter().filter(|m| !m.signer && !m.writable).cloned().collect();
    let mut ordered: Vec<Meta> = Vec::new();
    ordered.extend(signed_w);
    ordered.extend(signed_r);
    ordered.extend(unsigned_w);
    ordered.extend(unsigned_r);
    let index = |pk: &str, ordered: &[Meta]| {
        ordered
            .iter()
            .position(|m| m.pubkey == pk)
            .ok_or_else(|| format!("missing {pk}"))
    };
    let num_required_signatures = ordered.iter().filter(|m| m.signer).count();
    let num_readonly_signed = ordered.iter().filter(|m| m.signer && !m.writable).count();
    let num_readonly_unsigned = ordered.iter().filter(|m| !m.signer && !m.writable).count();

    let mut bytes = vec![
        num_required_signatures as u8,
        num_readonly_signed as u8,
        num_readonly_unsigned as u8,
    ];
    bytes.extend(compact_u16(ordered.len()));
    for m in &ordered {
        bytes.extend_from_slice(&decode_pk(&m.pubkey)?);
    }
    bytes.extend_from_slice(&decode_pk(recent_blockhash)?);
    bytes.extend(compact_u16(ixs.len()));
    for ix in ixs {
        bytes.push(index(&ix.program_id, &ordered)? as u8);
        bytes.extend(compact_u16(ix.keys.len()));
        for k in &ix.keys {
            bytes.push(index(&k.pubkey, &ordered)? as u8);
        }
        bytes.extend(compact_u16(ix.data.len()));
        bytes.extend_from_slice(&ix.data);
    }
    Ok(CompiledMessage {
        bytes,
        account_keys: ordered.iter().map(|m| m.pubkey.clone()).collect(),
        num_required_signatures,
    })
}

pub fn assemble_transaction(message: &[u8], signatures: &[[u8; 64]]) -> Vec<u8> {
    let mut out = compact_u16(signatures.len());
    for s in signatures {
        out.extend_from_slice(s);
    }
    out.extend_from_slice(message);
    out
}

fn b64_encode(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };
        s.push(T[(b0 >> 2) as usize] as char);
        s.push(T[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if i + 1 < data.len() {
            s.push(T[(((b1 & 0xf) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            s.push('=');
        }
        if i + 2 < data.len() {
            s.push(T[(b2 & 0x3f) as usize] as char);
        } else {
            s.push('=');
        }
        i += 3;
    }
    s
}

pub fn sign_and_send(rpc_url: &str, ixs: &[BuiltIx], signer: &KeypairSigner) -> Result<String, String> {
    let bh = rpc::latest_blockhash(rpc_url)?;
    let msg = compile_message(ixs, &signer.public_key(), &bh)?;
    if msg.num_required_signatures > 1 {
        return Err(format!(
            "this helper signs one key; message needs {}",
            msg.num_required_signatures
        ));
    }
    let sig = signer.sign(&msg.bytes);
    let tx = assemble_transaction(&msg.bytes, &[sig]);
    rpc::send_transaction_b64(rpc_url, &b64_encode(&tx))
}

pub fn simulate(rpc_url: &str, ixs: &[BuiltIx], fee_payer: &str) -> Result<Value, String> {
    let bh = rpc::latest_blockhash(rpc_url)?;
    let msg = compile_message(ixs, fee_payer, &bh)?;
    let zeros = [0u8; 64];
    let sigs: Vec<[u8; 64]> = (0..msg.num_required_signatures).map(|_| zeros).collect();
    let tx = assemble_transaction(&msg.bytes, &sigs);
    rpc::simulate_transaction_b64(rpc_url, &b64_encode(&tx))
}
