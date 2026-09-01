import * as bs58 from "bs58";
import { sha256 } from "@noble/hashes/sha256";
import { ed25519 } from "@noble/curves/ed25519";

export function encodePk(bytes: Uint8Array): string {
  return bs58.encode(bytes);
}

export function decodePk(s: string): Buffer {
  return Buffer.from(bs58.decode(s));
}

function isOnCurve(p: Uint8Array): boolean {
  try {
    ed25519.ExtendedPoint.fromHex(Buffer.from(p).toString("hex"));
    return true;
  } catch {
    return false;
  }
}

const PDA_MARKER = Buffer.from("ProgramDerivedAddress");

export function findProgramAddress(seeds: Uint8Array[], programId: Uint8Array): { address: string; bump: number } {
  const pid = Buffer.from(programId);
  for (let bump = 255; bump >= 0; bump--) {
    const parts = [...seeds.map((s) => Buffer.from(s)), Buffer.from([bump]), pid, PDA_MARKER];
    const hash = sha256(Buffer.concat(parts));
    if (!isOnCurve(hash)) return { address: encodePk(hash), bump };
  }
  throw new Error("unable to find PDA");
}
