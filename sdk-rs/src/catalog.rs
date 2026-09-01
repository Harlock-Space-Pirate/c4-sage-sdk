use serde::Deserialize;
use std::sync::OnceLock;

const IX_JSON: &str = include_str!("../../chain/programs/c4-sage-primary/instructions.json");
const ACC_JSON: &str = include_str!("../../chain/programs/c4-sage-primary/accounts.json");
const PDA_JSON: &str = include_str!("../../chain/pda-graph.json");
const REGISTRY_JSON: &str = include_str!("../../chain/registry.json");

#[derive(Debug, Deserialize, Clone)]
pub struct IxAccount {
    pub name: String,
    #[serde(default)]
    pub signer: bool,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub program: bool,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ArgGuess {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub offset: Option<u32>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InstructionDef {
    pub name: String,
    pub discriminator: String,
    pub track: String,
    #[serde(default)]
    pub accounts: Vec<IxAccount>,
    #[serde(rename = "argsGuess")]
    pub args_guess: Option<Vec<ArgGuess>>,
}

#[derive(Debug, Deserialize)]
struct InstructionsFile {
    #[serde(rename = "programId")]
    #[allow(dead_code)]
    pub program_id: String,
    pub instructions: Vec<InstructionDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub offset: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RegionDef {
    pub name: String,
    pub offset: u64,
    pub end: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountTypeDef {
    pub name: String,
    pub discriminator: String,
    #[serde(rename = "layoutComplete", default)]
    pub layout_complete: bool,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
    pub regions: Option<Vec<RegionDef>>,
    pub sample: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AccountsFile {
    #[serde(rename = "accountTypes")]
    pub account_types: Vec<AccountTypeDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PdaSeed {
    pub kind: String,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PdaDef {
    pub name: String,
    #[serde(rename = "programId")]
    pub program_id: Option<String>,
    #[serde(default)]
    pub seeds: Vec<PdaSeed>,
}

#[derive(Debug, Deserialize)]
struct PdaFile {
    #[serde(rename = "programId")]
    pub program_id: String,
    pub pdas: Vec<PdaDef>,
}

#[derive(Debug, Deserialize)]
struct RegistryProgram {
    id: String,
    #[serde(rename = "programId")]
    program_id: String,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    rpc: String,
    #[serde(rename = "gameAccount")]
    game_account: String,
    programs: Vec<RegistryProgram>,
}

pub struct Catalog {
    pub program_id: String,
    pub game: String,
    pub rpc: String,
    pub profile_program: String,
    pub faction_program: String,
    pub instructions: Vec<InstructionDef>,
    pub account_types: Vec<AccountTypeDef>,
    pub pdas: Vec<PdaDef>,
    pub pda_program_id: String,
}

fn load() -> Catalog {
    let ixs: InstructionsFile = serde_json::from_str(IX_JSON).expect("instructions.json");
    let acc: AccountsFile = serde_json::from_str(ACC_JSON).expect("accounts.json");
    let pdas: PdaFile = serde_json::from_str(PDA_JSON).expect("pda-graph.json");
    let reg: RegistryFile = serde_json::from_str(REGISTRY_JSON).expect("registry.json");
    let pid = |id: &str| {
        reg.programs
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.program_id.clone())
            .unwrap_or_default()
    };
    Catalog {
        program_id: pid("c4-sage-primary"),
        game: reg.game_account,
        rpc: reg.rpc,
        profile_program: pid("c4-profile"),
        faction_program: pid("c4-profile-faction"),
        instructions: ixs.instructions,
        account_types: acc.account_types,
        pdas: pdas.pdas,
        pda_program_id: pdas.program_id,
    }
}

static CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(load)
}

impl Catalog {
    pub fn instruction(&self, name: &str) -> Option<&InstructionDef> {
        self.instructions.iter().find(|i| i.name == name)
    }
    pub fn instruction_by_disc(&self, disc: &str) -> Option<&InstructionDef> {
        self.instructions.iter().find(|i| i.discriminator == disc)
    }
    pub fn account_type(&self, name: &str) -> Option<&AccountTypeDef> {
        self.account_types.iter().find(|t| t.name == name)
    }
    pub fn account_type_by_disc(&self, disc: &str) -> Option<&AccountTypeDef> {
        self.account_types.iter().find(|t| t.discriminator == disc)
    }
    pub fn pda(&self, name: &str) -> Option<&PdaDef> {
        self.pdas.iter().find(|p| p.name == name)
    }
}

pub fn field_offset(f: &FieldDef) -> Option<usize> {
    match &f.offset {
        Some(serde_json::Value::Number(n)) => n.as_u64().map(|x| x as usize),
        _ => None,
    }
}
