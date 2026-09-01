import { INSTRUCTIONS, type InstructionDef } from "./generated/instructions";
import { PROGRAM_ID } from "./generated/constants";

export type IxAccountMeta = { pubkey: string; isSigner: boolean; isWritable: boolean };
export type BuiltIx = { programId: string; keys: IxAccountMeta[]; data: Buffer };
export type NamedAccounts = Record<string, string>;

export function discBytes(hex: string): Buffer {
  return Buffer.from(hex, "hex");
}

export function encodeArgs(
  def: InstructionDef,
  args: Record<string, number | bigint | Uint8Array> | Uint8Array | undefined,
): Buffer {
  if (!args) return Buffer.alloc(0);
  if (args instanceof Uint8Array) return Buffer.from(args);
  const guesses = def.argsGuess;
  if (!guesses || guesses.length === 0) throw new Error(`${def.name}: no argsGuess; pass Uint8Array`);
  const parts: Buffer[] = [];
  for (const g of guesses) {
    const v = args[g.name];
    if (v === undefined) {
      if (g.type.startsWith("[u8;")) {
        parts.push(Buffer.alloc(Number(g.type.slice(4, -1))));
        continue;
      }
      throw new Error(`${def.name}: missing arg ${g.name}`);
    }
    if (v instanceof Uint8Array) {
      parts.push(Buffer.from(v));
      continue;
    }
    const n = typeof v === "bigint" ? v : BigInt(v);
    const width =
      g.type === "u8" || g.type === "bool" || g.type === "i8"
        ? 1
        : g.type === "u16" || g.type === "i16" || g.type === "shipId" || g.type === "cargoId" || g.type === "regionId" || g.type === "scanPatternId" || g.type === "recipeId"
          ? 2
          : g.type === "u32" || g.type === "i32"
            ? 4
            : g.type === "u64" || g.type === "i64"
              ? 8
              : 0;
    const b = Buffer.alloc(width);
    if (b.length === 0) throw new Error(`${def.name}: untyped arg ${g.name} (${g.type})`);
    if (b.length === 1) b.writeUInt8(Number(n));
    else if (b.length === 2) b.writeUInt16LE(Number(n));
    else if (b.length === 4) b.writeUInt32LE(Number(n));
    else b.writeBigUInt64LE(n);
    parts.push(b);
  }
  return Buffer.concat(parts);
}

export function buildIx(
  name: string,
  accounts: NamedAccounts,
  args?: Record<string, number | bigint | Uint8Array> | Uint8Array,
  programId = PROGRAM_ID,
): BuiltIx {
  const def = INSTRUCTIONS[name];
  if (!def) throw new Error(`unknown instruction ${name}`);
  const keys: IxAccountMeta[] = [];
  for (const a of def.accounts) {
    const pk = accounts[a.name];
    if (!pk) {
      if (a.optional) continue;
      throw new Error(`${name}: missing account ${a.name}`);
    }
    keys.push({ pubkey: pk, isSigner: a.signer, isWritable: a.writable });
  }
  return {
    programId,
    keys,
    data: Buffer.concat([discBytes(def.discriminator), encodeArgs(def, args)]),
  };
}
