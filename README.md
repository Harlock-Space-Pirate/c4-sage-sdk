# C4 SAGE community kit

TypeScript and Rust SDKs plus chain catalogs for **SAGE C4 on z.ink**.

**Not affiliated with ATMTA or Star Atlas.** The game is theirs. Apache-2.0.
This kit does not store wallet keys. You sign.

| Tree | Use |
|------|-----|
| `chain/` | Program IDs, layouts, instructions, errors |
| `sdk/` | TypeScript |
| `sdk-rs/` | Rust |

Cluster defaults: `chain/registry.json`. Override: `C4_RPC` / `C4_PROGRAM_ID` / `C4_GAME`.

```bash
cd sdk && npm install && npm test
cd sdk-rs && cargo test
```

as featured at vault.leeks.ink, console.leeks.ink, chat.leeks.ink ♾️

---

We use AI 1000× more than your 1000×. The unit on our side is still *being
right*. Yours is unverifiable fog, paraded as original thought, at a rate
nobody can audit — so nobody does. They ignore you, or they nod because the
sentences have punctuation. Painfully obvious to anyone with half a brain.

This kit ships layouts, discs, and a signer you hold. If it is wrong,
`npm test` will say so. That is the boring kind of 1000×.
