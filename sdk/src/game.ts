import { ACCOUNT_TYPES } from "./generated/accounts";
import { encodePk } from "./bytes";

export type ItemRow = { index: number; name: string; mint: string };
export type CategoryRow = { id: number; name: string };
export type GameHeader = {
  profile: string;
  atlasMint: string;
  atlasVault: string;
  atlasDaoVault: string;
  globalCeasefire: boolean;
};

function region(name: string) {
  const r = (ACCOUNT_TYPES.Game.regions ?? []).find((x) => x.name === name);
  if (!r) throw new Error(`Game region ${name} missing`);
  return r;
}

function off(name: string): number {
  const f = ACCOUNT_TYPES.Game.fields.find((x) => x.name === name);
  if (!f) throw new Error(`Game field ${name} missing`);
  return f.offset;
}

export function decodeGameHeader(data: Buffer): GameHeader {
  return {
    profile: encodePk(data.subarray(off("profile"), off("profile") + 32)),
    atlasMint: encodePk(data.subarray(off("currencies.atlas.mint"), off("currencies.atlas.mint") + 32)),
    atlasVault: encodePk(data.subarray(off("currencies.atlas.vault"), off("currencies.atlas.vault") + 32)),
    atlasDaoVault: encodePk(
      data.subarray(off("currencies.atlas.daoVault"), off("currencies.atlas.daoVault") + 32),
    ),
    globalCeasefire: data[off("globalCeasefire")] !== 0,
  };
}

export function decodeItemTable(data: Buffer, limit = 3640): ItemRow[] {
  const r = region("itemTable");
  const rec = 126;
  const n = Math.min(limit, Math.floor((r.end - r.offset) / rec));
  const out: ItemRow[] = [];
  for (let i = 0; i < n; i++) {
    const o = r.offset + i * rec;
    const name = data.subarray(o, o + 64).toString("utf8").replace(/\0+$/, "");
    if (!name) continue;
    out.push({ index: i, name, mint: encodePk(data.subarray(o + 64, o + 96)) });
  }
  return out;
}

export function decodeItemCategories(data: Buffer): CategoryRow[] {
  const r = region("itemCategories");
  const rec = 66;
  const n = Math.floor((r.end - r.offset) / rec);
  const out: CategoryRow[] = [];
  for (let i = 0; i < n; i++) {
    const o = r.offset + i * rec;
    const name = data.subarray(o, o + 64).toString("utf8").replace(/\0+$/, "");
    if (!name) break;
    out.push({ id: data.readUInt16LE(o + 64), name });
  }
  return out;
}

export function listShipConfigNames(data: Buffer): string[] {
  const r = region("shipConfigs");
  const slice = data.subarray(r.offset, r.end);
  const names: string[] = [];
  const needle = Buffer.from("Default Config");
  let from = 0;
  while (from < slice.length) {
    const at = slice.indexOf(needle, from);
    if (at < 0) break;
    let start = at;
    while (start > 0 && slice[start - 1] >= 32 && slice[start - 1] < 127 && slice[start - 1] !== 0) start--;
    names.push(slice.subarray(start, at + needle.length).toString("utf8"));
    from = at + needle.length;
  }
  return names;
}
