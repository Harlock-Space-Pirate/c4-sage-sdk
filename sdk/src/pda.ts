import { PDAS, type PdaDef } from "./generated/pdas";
import { PROGRAM_ID } from "./generated/constants";
import { decodePk, findProgramAddress } from "./bytes";

export type SeedInput = Record<string, Uint8Array | string>;

function seedBytes(def: PdaDef, input: SeedInput): Buffer[] {
  const out: Buffer[] = [];
  for (const s of def.seeds) {
    if (s.kind === "literal") {
      out.push(Buffer.from(s.value, "utf8"));
      continue;
    }
    if (s.kind === "literal_bytes") {
      if (s.value.includes("×")) {
        const n = Number(s.value.split("×")[1]) || 32;
        out.push(Buffer.alloc(n));
      } else if (s.value.startsWith("0x")) {
        out.push(Buffer.from(s.value.slice(2), "hex"));
      }
      continue;
    }
    const key = s.role.split(".")[0];
    const v = input[s.type] ?? input[key] ?? input[s.role];
    if (v === undefined) throw new Error(`PDA ${def.name}: missing seed ${s.type} (${s.role})`);
    out.push(typeof v === "string" ? decodePk(v) : Buffer.from(v));
  }
  return out;
}

export function findPda(name: string, input: SeedInput, programId = PROGRAM_ID) {
  const def = PDAS[name];
  if (!def) throw new Error(`unknown PDA ${name}`);
  const pid = decodePk(def.programId || programId);
  return findProgramAddress(seedBytes(def, input), pid);
}

export function starbasePlayerPda(starSystem: string, character: string) {
  return findPda("StarbasePlayer", { StarSystem: starSystem, Character: character });
}

export function characterPda(profile: string, game: string) {
  return findPda("Character", { Profile: profile, Game: game });
}

export function fleetPda(game: string, profile: string, label: string) {
  const label32 = Buffer.alloc(32);
  Buffer.from(label, "utf8").copy(label32);
  return findPda("Fleet", {
    Game: game,
    Profile: profile,
    "[u8;32]": label32,
    "fleet.label32": label32,
  });
}
