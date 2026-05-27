# Vendored codex — provenance

| Field | Value |
|-------|-------|
| Upstream | https://github.com/openai/codex |
| License | Apache-2.0 (see `LICENSE` + `NOTICE`) |
| Vendored commit | `0db49a7e6ae8fd459dd8d8d404086c748b9118a1` |
| Commit date | 2026-05-27 |
| Vendored on | 2026-05-28 |
| Scope | `codex-rs/` subtree only |

Only the `codex-rs/` Rust workspace is vendored — it carries the sandbox crates our spawn
bins reuse (`windows-sandbox-rs` → `run_windows_sandbox_capture`; `sandboxing` →
`create_seatbelt_command_args`; `protocol`). Shipped UNMODIFIED per Apache-2.0; no patches.

This replaces an earlier (~2026-05-22) snapshot that was copied WITHOUT recording the
upstream commit. The fresh re-vendor pins a known commit so the published sandbox bins are
reproducible from a fixed codex revision.

## Updating

```
git clone --depth 1 https://github.com/openai/codex /tmp/codex
# copy /tmp/codex/codex-rs over docs/refs/codex/codex-rs (mirror), + LICENSE + NOTICE
git -C /tmp/codex rev-parse HEAD   # <- bump "Vendored commit" above
```

Then bump the sandbox release tag (`v0.x.y`) and re-publish; downstream consumers (e.g.
VDG_CleanCode `build-sandbox.ps1`) re-pin the new release's SHA-256 set.
