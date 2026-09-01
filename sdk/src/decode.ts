import type { AccountTypeDef, FieldDef } from "./generated/accounts";
import { ACCOUNT_TYPES_BY_DISC } from "./generated/accounts";
import { encodePk } from "./bytes";

export type DecodedField = {
  name: string;
  type: string;
  offset: number;
  confidence: "certain" | "inferred";
  value: unknown;
};

function parseFixedBytes(t: string): number | null {
  const m = t.match(/^\[u8;(\d+)\]$/);
  return m ? Number(m[1]) : null;
}

export function decodeField(buf: Buffer, f: FieldDef): unknown {
  const o = f.offset;
  if (o >= buf.length) return undefined;
  const t = f.type;
  if (t === "Pubkey") {
    if (o + 32 > buf.length) return undefined;
    return encodePk(buf.subarray(o, o + 32));
  }
  if (t === "u8" || t === "u8/bool" || t === "bool") return buf[o];
  if (t === "u16") {
    if (o + 2 > buf.length) return undefined;
    return buf.readUInt16LE(o);
  }
  if (t === "u32") {
    if (o + 4 > buf.length) return undefined;
    return buf.readUInt32LE(o);
  }
  if (t === "u64") {
    if (o + 8 > buf.length) return undefined;
    return buf.readBigUInt64LE(o);
  }
  if (t === "i64") {
    if (o + 8 > buf.length) return undefined;
    return buf.readBigInt64LE(o);
  }
  const n = parseFixedBytes(t);
  if (n !== null) {
    if (o + n > buf.length) return buf.subarray(o);
    return buf.subarray(o, o + n);
  }
  if (t === "RemainingBytes") return buf.subarray(o);
  return buf.subarray(o);
}

export function identifyAccount(data: Buffer): AccountTypeDef | undefined {
  if (data.length < 8) return undefined;
  return ACCOUNT_TYPES_BY_DISC[data.subarray(0, 8).toString("hex")];
}

export function decodeAccount(data: Buffer, type?: AccountTypeDef) {
  const t = type ?? identifyAccount(data);
  const disc = data.subarray(0, 8).toString("hex");
  if (!t) return { type: "unknown", disc, fields: [] as DecodedField[] };
  return {
    type: t.name,
    disc,
    fields: t.fields.map((f) => ({
      name: f.name,
      type: f.type,
      offset: f.offset,
      confidence: f.confidence,
      value: decodeField(data, f),
    })),
  };
}

export function fieldMap(decoded: { fields: DecodedField[] }): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const f of decoded.fields) out[f.name] = f.value;
  return out;
}
