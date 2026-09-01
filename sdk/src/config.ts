import {
  PROGRAM_ID,
  GAME,
  RPC_DEFAULT,
  PROFILE_PROGRAM,
  FACTION_PROGRAM,
} from "./generated/constants";

/** One cluster endpoint + world. Defaults: chain/registry.json, then env, then constructor. */
export type ClusterConfig = {
  rpcUrl: string;
  programId: string;
  game: string;
  profileProgram: string;
  factionProgram: string;
};

export type ClusterOverrides = Partial<ClusterConfig>;

function env(name: string): string | undefined {
  return typeof process !== "undefined" ? process.env[name] : undefined;
}

/**
 * Resolve cluster settings. Precedence: explicit overrides > env > registry defaults.
 * Env: C4_RPC, C4_PROGRAM_ID, C4_GAME, C4_PROFILE_PROGRAM, C4_FACTION_PROGRAM.
 */
export function resolveConfig(overrides?: ClusterOverrides): ClusterConfig {
  return {
    rpcUrl: overrides?.rpcUrl ?? env("C4_RPC") ?? RPC_DEFAULT,
    programId: overrides?.programId ?? env("C4_PROGRAM_ID") ?? PROGRAM_ID,
    game: overrides?.game ?? env("C4_GAME") ?? GAME,
    profileProgram: overrides?.profileProgram ?? env("C4_PROFILE_PROGRAM") ?? PROFILE_PROGRAM,
    factionProgram: overrides?.factionProgram ?? env("C4_FACTION_PROGRAM") ?? FACTION_PROGRAM,
  };
}
