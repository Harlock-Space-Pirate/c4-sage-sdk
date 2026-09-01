use crate::bytes::encode_pk;
use crate::catalog::{field_offset, AccountTypeDef, FieldDef};

#[derive(Debug, Clone)]
pub struct DecodedField {
    pub name: String,
    pub ty: String,
    pub offset: usize,
    pub confidence: String,
    pub value: DecodedValue,
}

#[derive(Debug, Clone)]
pub enum DecodedValue {
    Pubkey(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I64(i64),
    Bytes(Vec<u8>),
    Missing,
}

pub fn identify_account<'a>(data: &[u8], types: &'a [AccountTypeDef]) -> Option<&'a AccountTypeDef> {
    if data.len() < 8 {
        return None;
    }
    let disc = hex::encode(&data[..8]);
    types.iter().find(|t| t.discriminator == disc)
}

mod hex {
    pub fn encode(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}

pub fn decode_field(data: &[u8], f: &FieldDef) -> DecodedValue {
    let Some(o) = field_offset(f) else {
        return DecodedValue::Missing;
    };
    if o >= data.len() {
        return DecodedValue::Missing;
    }
    let t = f.ty.as_str();
    if t == "Pubkey" {
        if o + 32 > data.len() {
            return DecodedValue::Missing;
        }
        return DecodedValue::Pubkey(encode_pk(&data[o..o + 32]));
    }
    if t == "u8" || t == "u8/bool" || t == "bool" {
        return DecodedValue::U8(data[o]);
    }
    if t == "u16" && o + 2 <= data.len() {
        return DecodedValue::U16(u16::from_le_bytes(data[o..o + 2].try_into().unwrap()));
    }
    if t == "u32" && o + 4 <= data.len() {
        return DecodedValue::U32(u32::from_le_bytes(data[o..o + 4].try_into().unwrap()));
    }
    if t == "u64" && o + 8 <= data.len() {
        return DecodedValue::U64(u64::from_le_bytes(data[o..o + 8].try_into().unwrap()));
    }
    if t == "i64" && o + 8 <= data.len() {
        return DecodedValue::I64(i64::from_le_bytes(data[o..o + 8].try_into().unwrap()));
    }
    if let Some(n) = t.strip_prefix("[u8;").and_then(|s| s.strip_suffix(']')).and_then(|s| s.parse::<usize>().ok()) {
        let end = (o + n).min(data.len());
        return DecodedValue::Bytes(data[o..end].to_vec());
    }
    if t == "RemainingBytes" {
        return DecodedValue::Bytes(data[o..].to_vec());
    }
    DecodedValue::Bytes(data[o..].to_vec())
}

pub fn decode_account(data: &[u8], ty: &AccountTypeDef) -> Vec<DecodedField> {
    ty.fields
        .iter()
        .filter_map(|f| {
            let o = field_offset(f)?;
            Some(DecodedField {
                name: f.name.clone(),
                ty: f.ty.clone(),
                offset: o,
                confidence: f.confidence.clone(),
                value: decode_field(data, f),
            })
        })
        .collect()
}
