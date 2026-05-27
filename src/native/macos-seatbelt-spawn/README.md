# vdg-macos-seatbelt-spawn (#472-E E-3)

Our standalone bin that runs a command under codex's **macOS seatbelt** sandbox. It **depends on** the
vendored codex crates (`codex-sandboxing` + `codex-protocol`) and calls their public
`create_seatbelt_command_args` + `PermissionProfile::to_runtime_permissions` — it does **not** modify
any codex source. A C# agent spawns this bin exactly like `codex-linux-sandbox`, passing the SAME
`--permission-profile` JSON.

## How it works
Parse `--sandbox-policy-cwd / --command-cwd / --permission-profile <json> / -- <cmd>` → deserialize the
PermissionProfile → `to_runtime_permissions()` → `create_seatbelt_command_args(...)` → prepend
`/usr/bin/sandbox-exec` → `exec` (mirrors `codex-linux-sandbox` exec'ing bwrap; the command's stdio +
exit code flow straight to the agent).

## Enforcement boundary (seatbelt)
- **Filesystem write**: BLOCKED outside the workspace (writable = workspace + /tmp + $TMPDIR).
- **Network**: DENIED (seatbelt `(deny network*)` — unlike Windows non-admin, seatbelt blocks network).

Seatbelt enforces both write-confinement and network-block in one profile, so (like Linux) no
network-gap disclosure is needed.

## Why the [patch] block + Cargo.lock are copied from codex
This crate lives **outside** the codex-rs workspace, so it does not inherit codex-rs's `[patch]` set or
its `Cargo.lock`. To resolve **identically** to codex while keeping codex pristine:
- `[patch.crates-io]` (+ the ssh patch) in `Cargo.toml` are copied **verbatim** from
  `docs/refs/codex/codex-rs/Cargo.toml` — do not invent revs.
- `Cargo.lock` is copied from `docs/refs/codex/codex-rs/Cargo.lock`.

## Procedure when the vendored codex ref is bumped
Every codex bump requires a re-sync + re-verify:
1. Re-copy `[patch]` blocks from `docs/refs/codex/codex-rs/Cargo.toml`.
2. Re-copy `docs/refs/codex/codex-rs/Cargo.lock` → this crate's `Cargo.lock`.
3. **No local mac**: build + run the enforce round-trip (write-inside OK / write-outside BLOCKED /
   network BLOCKED) on the **publish CI mac runner** — compile success alone does NOT prove enforcement.
