use crate::catalog::catalog;

/// Cluster endpoint + world. Change defaults in `chain/registry.json`.
/// Runtime: constructor overrides > env > registry.
#[derive(Clone, Debug)]
pub struct ClusterConfig {
    pub rpc_url: String,
    pub program_id: String,
    pub game: String,
    pub profile_program: String,
    pub faction_program: String,
}

#[derive(Clone, Debug, Default)]
pub struct ClusterOverrides {
    pub rpc_url: Option<String>,
    pub program_id: Option<String>,
    pub game: Option<String>,
    pub profile_program: Option<String>,
    pub faction_program: Option<String>,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Env: `C4_RPC`, `C4_PROGRAM_ID`, `C4_GAME`, `C4_PROFILE_PROGRAM`, `C4_FACTION_PROGRAM`.
pub fn resolve_config(over: ClusterOverrides) -> ClusterConfig {
    let c = catalog();
    ClusterConfig {
        rpc_url: over.rpc_url.or_else(|| env("C4_RPC")).unwrap_or_else(|| c.rpc.clone()),
        program_id: over
            .program_id
            .or_else(|| env("C4_PROGRAM_ID"))
            .unwrap_or_else(|| c.program_id.clone()),
        game: over.game.or_else(|| env("C4_GAME")).unwrap_or_else(|| c.game.clone()),
        profile_program: over
            .profile_program
            .or_else(|| env("C4_PROFILE_PROGRAM"))
            .unwrap_or_else(|| c.profile_program.clone()),
        faction_program: over
            .faction_program
            .or_else(|| env("C4_FACTION_PROGRAM"))
            .unwrap_or_else(|| c.faction_program.clone()),
    }
}
