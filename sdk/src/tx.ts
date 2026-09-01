/** Legacy Solana transaction compile / sign / send. The SDK never stores keys. */

import { ed25519 } from "@noble/curves/ed25519";
import bs58 from "bs58";
import type { Rpc } from "./rpc";
import type { BuiltIx } from "./ix";

export type Signer = {
  publicKey: string;
  sign: (message: Uint8Array) => Uint8Array | Promise<Uint8Array>;
};

/** 32-byte seed or 64-byte Solana secret. Headless / lancer wallets only — never a main wallet. */
export function keypairSigner(secretKey: Uint8Array): Signer {
  const seed = secretKey.length >= 64 ? secretKey.subarray(0, 32) : secretKey;
  if (seed.length !== 32) throw new Error("secret key must be 32 or 64 bytes");
  const pub = ed25519.getPublicKey(seed);
  return {
    publicKey: bs58.encode(pub),
    sign: (msg) => ed25519.sign(msg, seed),
  };
}

function compactU16(n: number): Buffer {
  if (n < 0x80) return Buffer.from([n]);
  if (n < 0x4000) return Buffer.from([(n & 0x7f) | 0x80, n >> 7]);
  return Buffer.from([(n & 0x7f) | 0x80, ((n >> 7) & 0x7f) | 0x80, n >> 14]);
}

function pk32(s: string): Buffer {
  const b = Buffer.from(bs58.decode(s));
  if (b.length !== 32) throw new Error(`not a pubkey: ${s}`);
  return b;
}

export type CompiledMessage = {
  bytes: Buffer;
  accountKeys: string[];
  numRequiredSignatures: number;
};

/** Compile a legacy unsigned message. feePayer is the first signer. */
export function compileMessage(ixs: BuiltIx[], feePayer: string, recentBlockhash: string): CompiledMessage {
  type Meta = { pubkey: string; signer: boolean; writable: boolean };
  const map = new Map<string, Meta>();
  const bump = (pk: string, signer: boolean, writable: boolean) => {
    const cur = map.get(pk) ?? { pubkey: pk, signer: false, writable: false };
    cur.signer = cur.signer || signer;
    cur.writable = cur.writable || writable;
    map.set(pk, cur);
  };
  bump(feePayer, true, true);
  for (const ix of ixs) {
    bump(ix.programId, false, false);
    for (const k of ix.keys) bump(k.pubkey, k.isSigner, k.isWritable);
  }
  const signedW = [...map.values()].filter((m) => m.signer && m.writable);
  const signedR = [...map.values()].filter((m) => m.signer && !m.writable);
  const unsignedW = [...map.values()].filter((m) => !m.signer && m.writable);
  const unsignedR = [...map.values()].filter((m) => !m.signer && !m.writable);
  const ordered = [
    ...signedW.filter((m) => m.pubkey === feePayer),
    ...signedW.filter((m) => m.pubkey !== feePayer),
    ...signedR,
    ...unsignedW,
    ...unsignedR,
  ];
  const index = new Map(ordered.map((m, i) => [m.pubkey, i]));
  const numRequiredSignatures = signedW.length + signedR.length;
  const numReadonlySigned = signedR.length;
  const numReadonlyUnsigned = unsignedR.length;

  const parts: Buffer[] = [
    Buffer.from([numRequiredSignatures, numReadonlySigned, numReadonlyUnsigned]),
    compactU16(ordered.length),
    ...ordered.map((m) => pk32(m.pubkey)),
    pk32(recentBlockhash),
    compactU16(ixs.length),
  ];
  for (const ix of ixs) {
    const prog = index.get(ix.programId);
    if (prog === undefined) throw new Error("program id missing from message");
    const accIdx = ix.keys.map((k) => {
      const i = index.get(k.pubkey);
      if (i === undefined) throw new Error(`account ${k.pubkey} missing`);
      return i;
    });
    parts.push(
      Buffer.from([prog]),
      compactU16(accIdx.length),
      Buffer.from(accIdx),
      compactU16(ix.data.length),
      Buffer.from(ix.data),
    );
  }
  return {
    bytes: Buffer.concat(parts),
    accountKeys: ordered.map((m) => m.pubkey),
    numRequiredSignatures,
  };
}

export function assembleTransaction(message: Buffer, signatures: Uint8Array[]): Buffer {
  const parts: Buffer[] = [compactU16(signatures.length)];
  for (const s of signatures) {
    if (s.length !== 64) throw new Error("signature must be 64 bytes");
    parts.push(Buffer.from(s));
  }
  parts.push(message);
  return Buffer.concat(parts);
}

export async function signMessage(message: Buffer, signers: Signer[]): Promise<Uint8Array[]> {
  const out: Uint8Array[] = [];
  for (const s of signers) {
    const sig = await s.sign(message);
    out.push(sig instanceof Uint8Array ? sig : new Uint8Array(sig));
  }
  return out;
}

export type SimResult = { err: unknown; logs: string[]; units?: number };

export async function simulateIxs(
  rpc: Rpc,
  ixs: BuiltIx[],
  feePayer: string,
  signers: Signer[],
): Promise<SimResult> {
  const latest = (await rpc("getLatestBlockhash", [{ commitment: "processed" }])) as {
    value: { blockhash: string };
  };
  const msg = compileMessage(ixs, feePayer, latest.value.blockhash);
  const sigs = await signMessage(msg.bytes, signers.slice(0, msg.numRequiredSignatures));
  while (sigs.length < msg.numRequiredSignatures) sigs.push(new Uint8Array(64));
  const tx = assembleTransaction(msg.bytes, sigs);
  const val = (await rpc("simulateTransaction", [
    tx.toString("base64"),
    {
      encoding: "base64",
      sigVerify: false,
      replaceRecentBlockhash: true,
      commitment: "processed",
    },
  ])) as { value: { err: unknown; logs?: string[]; unitsConsumed?: number } };
  return { err: val.value.err, logs: val.value.logs ?? [], units: val.value.unitsConsumed };
}

/** Zink public RPCs: legacy sendTransaction, skipPreflight. Caller already signed. */
export async function sendSignedTransaction(rpc: Rpc, tx: Uint8Array): Promise<string> {
  const sig = (await rpc("sendTransaction", [
    Buffer.from(tx).toString("base64"),
    { encoding: "base64", skipPreflight: true },
  ])) as string;
  return sig;
}

export async function signAndSend(
  rpc: Rpc,
  ixs: BuiltIx[],
  signers: Signer[],
): Promise<string> {
  if (!signers.length) throw new Error("need at least one signer (fee payer)");
  const latest = (await rpc("getLatestBlockhash", [{ commitment: "confirmed" }])) as {
    value: { blockhash: string };
  };
  const msg = compileMessage(ixs, signers[0].publicKey, latest.value.blockhash);
  if (signers.length < msg.numRequiredSignatures) {
    throw new Error(`need ${msg.numRequiredSignatures} signers, got ${signers.length}`);
  }
  const sigs = await signMessage(msg.bytes, signers.slice(0, msg.numRequiredSignatures));
  const tx = assembleTransaction(msg.bytes, sigs);
  return sendSignedTransaction(rpc, tx);
}
