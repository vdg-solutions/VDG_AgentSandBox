# vdg-windows-restricted-spawn (#472-E E-2a)

Our standalone bin that runs a command under codex's **non-admin (RestrictedToken)** Windows sandbox.
It **depends on** the vendored codex crate (`codex-windows-sandbox`) and calls its public
`run_windows_sandbox_capture` — it does **not** modify any codex source. A C# agent spawns this bin
exactly like `codex-linux-sandbox`.

## Enforcement boundary (non-admin RestrictedToken)
- **Filesystem write**: BLOCKED outside the workspace (WRITE_RESTRICTED token + deny-write ACLs).
- **Filesystem read**: NOT restricted (deny-read requires the elevated backend).
- **Network**: NOT restricted (WFP requires the elevated backend).

The report surfaced to the user MUST disclose the network gap (see #472-E follow-up).

## Why the [patch] block + Cargo.lock are copied from codex
This crate lives **outside** the codex-rs workspace, so it does not inherit codex-rs's `[patch]` set
or its `Cargo.lock`. Building standalone would re-resolve transitive deps freshly and diverge
(tokio-tungstenite `proxy` feature; multiple `hashbrown` versions → starlark_map/allocative trait
conflict). To resolve **identically** to codex while keeping codex pristine:
- `[patch.crates-io]` (+ the ssh patch) in `Cargo.toml` are copied **verbatim** from
  `docs/refs/codex/codex-rs/Cargo.toml` — do not invent revs.
- `Cargo.lock` is copied from `docs/refs/codex/codex-rs/Cargo.lock` (cargo prunes it to this crate's
  subset on first build; the pruned lock is committed for reproducibility).

## Procedure when the vendored codex ref is bumped
Because this bin couples to codex internals (the public fn + the pinned version graph), every codex
bump requires a re-sync + re-verify:
1. Re-copy `[patch]` blocks from `docs/refs/codex/codex-rs/Cargo.toml`.
2. Re-copy `docs/refs/codex/codex-rs/Cargo.lock` → this crate's `Cargo.lock`.
3. Rebuild on Windows + re-run the enforce round-trip (write-inside-workspace OK / write-outside
   BLOCKED) — compile success alone does NOT prove enforcement.
