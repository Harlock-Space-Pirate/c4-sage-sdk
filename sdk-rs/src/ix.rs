use crate::catalog::{catalog, InstructionDef};

#[derive(Debug, Clone)]
pub struct IxAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug, Clone)]
pub struct BuiltIx {
    pub program_id: String,
    pub keys: Vec<IxAccountMeta>,
    pub data: Vec<u8>,
}

pub fn disc_bytes(hex: &str) -> Result<Vec<u8>, String> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

pub fn encode_args(def: &InstructionDef, args: &[(&str, ArgVal)]) -> Result<Vec<u8>, String> {
    let Some(guesses) = &def.args_guess else {
        if args.is_empty() {
            return Ok(vec![]);
        }
        return Err(format!("{}: no argsGuess; pass raw bytes", def.name));
    };
    let mut out = Vec::new();
    for g in guesses {
        let v = args.iter().find(|(n, _)| *n == g.name).map(|(_, v)| v);
        match v {
            None if g.ty.starts_with("[u8;") => {
                let n: usize = g.ty[4..g.ty.len() - 1].parse().unwrap_or(0);
                out.extend(std::iter::repeat(0u8).take(n));
            }
            None => return Err(format!("{}: missing arg {}", def.name, g.name)),
            Some(ArgVal::Raw(b)) => out.extend_from_slice(b),
            Some(ArgVal::U64(n)) => match g.ty.as_str() {
                "u8" => out.push(*n as u8),
                "u16" => out.extend_from_slice(&(*n as u16).to_le_bytes()),
                "u32" => out.extend_from_slice(&(*n as u32).to_le_bytes()),
                "u64" => out.extend_from_slice(&n.to_le_bytes()),
                _ => return Err(format!("{}: untyped arg {} ({})", def.name, g.name, g.ty)),
            },
        }
    }
    Ok(out)
}

pub enum ArgVal<'a> {
    U64(u64),
    Raw(&'a [u8]),
}

pub fn build_ix(
    name: &str,
    accounts: &[(&str, &str)],
    args: &[(&str, ArgVal)],
    program_id: Option<&str>,
) -> Result<BuiltIx, String> {
    let c = catalog();
    let def = c.instruction(name).ok_or_else(|| format!("unknown instruction {name}"))?;
    let keys: Vec<IxAccountMeta> = def
        .accounts
        .iter()
        .map(|a| {
            let pk = accounts.iter().find(|(n, _)| *n == a.name).map(|(_, p)| (*p).to_string());
            match pk {
                None if a.optional => Ok(None),
                None => Err(format!("{name}: missing account {}", a.name)),
                Some(pubkey) => Ok(Some(IxAccountMeta {
                    pubkey,
                    is_signer: a.signer,
                    is_writable: a.writable,
                })),
            }
        })
        .collect::<Result<Vec<Option<IxAccountMeta>>, String>>()?
        .into_iter()
        .flatten()
        .collect();
    let mut data = disc_bytes(&def.discriminator)?;
    data.extend(encode_args(def, args)?);
    Ok(BuiltIx {
        program_id: program_id.unwrap_or(&c.program_id).to_string(),
        keys,
        data,
    })
}
