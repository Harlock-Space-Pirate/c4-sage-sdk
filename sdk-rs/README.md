# C4 SAGE community SDK (Rust)

**Not affiliated with ATMTA or Star Atlas.** Community toolkit for SAGE C4 on
z.ink. Apache-2.0. You sign; this crate does not store keys.

```bash
cd sdk-rs && cargo test
cargo run --example headless_read
```

Cluster defaults: `chain/registry.json`. Env: `C4_RPC` / `C4_PROGRAM_ID` / `C4_GAME`.

Write (throwaway key only):

```rust
let signer = KeypairSigner::from_secret(&secret)?;
let ix = client.build_ix("withdraw_atlas", &[/* names */], &[/* args */])?;
let _sim = client.simulate(&[ix.clone()], &signer.public_key())?;
// client.sign_and_send(&[ix], &signer)?; // you opt in
```

Zink public RPC: `sendTransaction` + `skipPreflight: true`.
