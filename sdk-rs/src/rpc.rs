//! JSON-RPC to a Zink endpoint. No keys.

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct RpcEnvelope {
    result: Option<Value>,
    error: Option<Value>,
}

pub fn rpc(url: &str, method: &str, params: Value) -> Result<Value, String> {
    let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
    let resp = ureq::post(url)
        .set("content-type", "application/json")
        .send_json(body)
        .map_err(|e| e.to_string())?;
    let env: RpcEnvelope = resp.into_json().map_err(|e| e.to_string())?;
    if let Some(err) = env.error {
        return Err(err.to_string());
    }
    env.result.ok_or_else(|| "empty RPC result".into())
}

pub fn get_account_data(url: &str, pubkey: &str) -> Result<Option<Vec<u8>>, String> {
    let v = rpc(
        url,
        "getAccountInfo",
        serde_json::json!([pubkey, {"encoding":"base64"}]),
    )?;
    let data = v
        .pointer("/value/data/0")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    match data {
        None => Ok(None),
        Some(b64) => Ok(Some(b64_decode(&b64)?)),
    }
}

fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 3 < bytes.len() {
        let a = val(bytes[i]).unwrap_or(0);
        let b = val(bytes[i + 1]).unwrap_or(0);
        let c = val(bytes[i + 2]).unwrap_or(0);
        let d = val(bytes[i + 3]).unwrap_or(0);
        out.push((a << 2) | (b >> 4));
        if bytes[i + 2] != b'=' {
            out.push(((b & 0xf) << 4) | (c >> 2));
        }
        if bytes[i + 3] != b'=' {
            out.push(((c & 0x3) << 6) | d);
        }
        i += 4;
    }
    Ok(out)
}

pub fn latest_blockhash(url: &str) -> Result<String, String> {
    let v = rpc(
        url,
        "getLatestBlockhash",
        serde_json::json!([{"commitment":"confirmed"}]),
    )?;
    v.pointer("/value/blockhash")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "no blockhash".into())
}

pub fn send_transaction_b64(url: &str, tx_b64: &str) -> Result<String, String> {
    let v = rpc(
        url,
        "sendTransaction",
        serde_json::json!([tx_b64, {"encoding":"base64","skipPreflight":true}]),
    )?;
    v.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "sendTransaction: expected signature string".into())
}

pub fn simulate_transaction_b64(url: &str, tx_b64: &str) -> Result<Value, String> {
    rpc(
        url,
        "simulateTransaction",
        serde_json::json!([
            tx_b64,
            {
                "encoding": "base64",
                "sigVerify": false,
                "replaceRecentBlockhash": true,
                "commitment": "processed"
            }
        ]),
    )
}
