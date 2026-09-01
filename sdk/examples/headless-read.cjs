#!/usr/bin/env node
/** Read-only: fetch the Game header. No keys. */
const { C4SageClient } = require("../dist/index.js");

async function main() {
  const client = new C4SageClient();
  const g = await client.fetchGame();
  if (!g) {
    console.error("Game account missing on", client.config.rpcUrl);
    process.exit(1);
  }
  console.log({
    rpc: client.config.rpcUrl,
    bytes: g.bytes,
    ceasefire: g.header.globalCeasefire,
    atlasMint: g.header.atlasMint,
    itemsPreview: g.items.slice(0, 3),
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
