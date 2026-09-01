# C4 SAGE community SDK (TypeScript)

**Not affiliated with ATMTA or Star Atlas.** The game is theirs. This kit is a
community dictionary + readers + instruction builder for **SAGE C4 on z.ink**,
so others can build dashboards, UIs, and headless bots. We do not store wallet
keys. You sign.

Apache-2.0. Do not mix with Solana-mainnet SAGE Starbased (`@staratlas/sage`).

## Install

Not on npmjs. Clone this repo, then this folder plus `../chain`.
Cluster: `chain/registry.json`.
Override: `C4_RPC` / `C4_PROGRAM_ID` / `C4_GAME` (rpc1 or rpc2).

```bash
cd sdk && npm install && npm test
```

## Read

```js
const { C4SageClient } = require("./dist");
const client = new C4SageClient();
const fleet = await client.fetchFleet("YourFleetPubkey…");
```

`node examples/headless-read.cjs`

## Write (you sign)

Optional certificate accounts may be omitted. Then either:

- **Headless:** `keypairSigner(secretBytes)` + `client.signAndSend(ixs, [signer])`
  Throwaway key only.
- **Browser / own UI:** `compileMessage` → wallet `signMessage` / `signTransaction` →
  `sendTransaction` base64 `skipPreflight: true` (Zink public RPCs).
  See `examples/browser-wallet.md`.

Simulate first: `client.simulate(ixs, feePayer, signers)`.

## What this is not

A full gameplay robot. Loops (mine → warp → repeat) live in **your** app.
This kit is the C4 language: decode, PDAs, build bytes, optional sign+send.
