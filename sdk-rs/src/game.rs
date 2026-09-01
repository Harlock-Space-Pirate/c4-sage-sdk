use crate::bytes::encode_pk;
use crate::catalog::catalog;

#[derive(Debug, Clone)]
pub struct GameHeader {
    pub profile: String,
    pub atlas_mint: String,
    pub atlas_vault: String,
    pub atlas_dao_vault: String,
    pub global_ceasefire: bool,
}

#[derive(Debug, Clone)]
pub struct ItemRow {
    pub index: usize,
    pub name: String,
    pub mint: String,
}

#[derive(Debug, Clone)]
pub struct CategoryRow {
    pub id: u16,
    pub name: String,
}

fn region(name: &str) -> Result<(u64, u64), String> {
    let g = catalog().account_type("Game").ok_or("Game type missing")?;
    let r = g
        .regions
        .as_ref()
        .and_then(|rs| rs.iter().find(|r| r.name == name))
        .ok_or_else(|| format!("Game region {name} missing"))?;
    Ok((r.offset, r.end))
}

fn off(name: &str) -> Result<usize, String> {
    let g = catalog().account_type("Game").ok_or("Game type missing")?;
    let f = g
        .fields
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("Game field {name} missing"))?;
    let o = f
        .offset
        .as_ref()
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("Game field {name} has no offset"))?;
    Ok(o as usize)
}

pub fn decode_game_header(data: &[u8]) -> Result<GameHeader, String> {
    let cease = off("globalCeasefire")?;
    if data.len() < cease + 1 {
        return Err("game header too small".into());
    }
    let profile = off("profile")?;
    let mint = off("currencies.atlas.mint")?;
    let vault = off("currencies.atlas.vault")?;
    let dao = off("currencies.atlas.daoVault")?;
    Ok(GameHeader {
        profile: encode_pk(&data[profile..profile + 32]),
        atlas_mint: encode_pk(&data[mint..mint + 32]),
        atlas_vault: encode_pk(&data[vault..vault + 32]),
        atlas_dao_vault: encode_pk(&data[dao..dao + 32]),
        global_ceasefire: data[cease] != 0,
    })
}

pub fn decode_item_table(data: &[u8], limit: usize) -> Result<Vec<ItemRow>, String> {
    let (start, end) = region("itemTable")?;
    let rec = 126usize;
    let n = ((end - start) as usize / rec).min(limit);
    let mut out = Vec::new();
    for i in 0..n {
        let o = start as usize + i * rec;
        if o + 96 > data.len() {
            break;
        }
        let name = String::from_utf8_lossy(&data[o..o + 64])
            .trim_end_matches('\0')
            .to_string();
        if name.is_empty() {
            continue;
        }
        out.push(ItemRow {
            index: i,
            name,
            mint: encode_pk(&data[o + 64..o + 96]),
        });
    }
    Ok(out)
}

pub fn decode_item_categories(data: &[u8]) -> Result<Vec<CategoryRow>, String> {
    let (start, end) = region("itemCategories")?;
    let rec = 66usize;
    let n = (end - start) as usize / rec;
    let mut out = Vec::new();
    for i in 0..n {
        let o = start as usize + i * rec;
        if o + 66 > data.len() {
            break;
        }
        let name = String::from_utf8_lossy(&data[o..o + 64])
            .trim_end_matches('\0')
            .to_string();
        if name.is_empty() {
            break;
        }
        out.push(CategoryRow {
            id: u16::from_le_bytes(data[o + 64..o + 66].try_into().unwrap()),
            name,
        });
    }
    Ok(out)
}
