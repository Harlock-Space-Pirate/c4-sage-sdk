# Chain catalogs

C4 SAGE on Zink (`https://rpc1.z.ink` / `https://rpc2.z.ink`).
Star Frame discriminators: `sha256("global:"+name)[:8]` / `sha256("account:"+Type)[:8]`.
Offsets match live accounts.

UI world: `c4-sage-primary` (`C4SAge…`). Do not mix with Solana-mainnet SAGE Starbased.

- `registry.json` — RPC, Game, program ids
- `programs/<id>/instructions.json` — instructions
- `programs/<id>/accounts.json` — account layouts
- `programs/<id>/errors.json` — program errors

Clients: TypeScript `../sdk/`, Rust `../sdk-rs/`. Override cluster via `C4_RPC` / `C4_PROGRAM_ID` / `C4_GAME`.
