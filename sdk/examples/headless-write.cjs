#!/usr/bin/env node
/**
 * Headless write shape. YOU sign. The SDK never stores the key.
 * Lancer / throwaway key only (same idea as SLY standalone).
 *
 *   C4_SECRET_KEY_JSON='[...64 numbers...]' node examples/headless-write.cjs
 *
 * Default: SIMULATE only. C4_SEND=1 actually submits (your key, your risk).
 */
const { C4SageClient, keypairSigner, buildIx } = require("../dist/index.js");

async function main() {
  const raw = process.env.C4_SECRET_KEY_JSON;
  if (!raw) {
    console.error("Set C4_SECRET_KEY_JSON to a JSON array of 32 or 64 bytes. Lancer wallet only.");
    process.exit(1);
  }
  const signer = keypairSigner(Uint8Array.from(JSON.parse(raw)));
  const client = new C4SageClient();
  const ix = buildIx(
    "withdraw_atlas",
    {
      profileValidationSigner: signer.publicKey,
      profileValidationProfile: process.env.C4_PROFILE || signer.publicKey,
      profileValidationProgram: client.config.profileProgram,
      character: process.env.C4_CHARACTER || signer.publicKey,
      game: client.game,
      gameAtlasVault: client.game,
      destination: client.game,
      currencyCache: client.game,
      tokenProgram: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    },
    { keyIndex: 0, amount: 1n },
  );
  const sim = await client.simulate([ix], signer.publicKey, [signer]);
  console.log({ feePayer: signer.publicKey, err: sim.err, logs: sim.logs.slice(-6) });
  if (process.env.C4_SEND === "1") {
    console.log("sent", await client.signAndSend([ix], [signer]));
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
