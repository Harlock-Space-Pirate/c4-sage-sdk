# Browser / own UI

The SDK builds instruction bytes. Your page asks the **user's wallet** to sign.
We never see the secret key.

This kit is the GitHub tree, **not** the npm registry. `@staratlas-app/c4-sage-sdk` is only the name in this folder’s `package.json`. There is no npmjs package under that name. (`@staratlas/sage` on npm is Solana SAGE, a different program.)

1. From this repo: `cd sdk && npm install && npm run build`  
   (from your own app: `npm install /path/to/c4-sage-sdk/sdk` so that local name resolves)
2. Point the wallet at the **Zink** network (`https://rpc1.z.ink` or `https://rpc2.z.ink`), not Solana mainnet.
3. Build, then hand the compiled transaction to the wallet:

```js
import { C4SageClient, compileMessage, assembleTransaction } from "@staratlas-app/c4-sage-sdk";
// after `npm install /path/to/c4-sage-sdk/sdk` — or import from this repo’s `./dist`

const client = new C4SageClient();
const ix = client.buildIx("withdraw_atlas", { /* named accounts */ }, { keyIndex: 0, amount: 1n });

const { value } = await client.rpc("getLatestBlockhash", [{ commitment: "confirmed" }]);
const msg = compileMessage([ix], wallet.publicKey, value.blockhash);

// Wallet adapter (Phantom / Backpack / etc. on a custom cluster):
const sig = await wallet.signMessage(msg.bytes); // or their signTransaction hook
const tx = assembleTransaction(msg.bytes, [sig]);
const signature = await client.rpc("sendTransaction", [
  tx.toString("base64"),
  { encoding: "base64", skipPreflight: true },
]);
```

Headless bots (SLY-style) use `keypairSigner` + `signAndSend` instead of a browser wallet.
Keep that key off your main funds.
