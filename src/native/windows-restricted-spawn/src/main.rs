// VDG #472-E E-2a — runs a command under codex's non-admin (RestrictedToken) Windows sandbox.
// Depends on the vendored codex crate via its PUBLIC API (run_windows_sandbox_capture); it does NOT
// modify any codex source. FS-write is confined to the workspace (WRITE_RESTRICTED token + deny-write
// ACLs); reads + network are NOT restricted (those need the elevated backend). A C# agent spawns this
// bin like codex-linux-sandbox.
//
// CLI: --policy <preset|json> --sandbox-policy-cwd <dir> [--command-cwd <dir>] [--codex-home <dir>]
//      [--timeout-ms <n>] -- <command> [args...]

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use std::collections::HashMap;
    use std::io::Write;
    use std::path::PathBuf;

    use anyhow::{bail, Context};
    use codex_windows_sandbox::{
        run_windows_sandbox_capture, run_windows_sandbox_capture_elevated, ElevatedSandboxCaptureRequest,
    };

    let args: Vec<String> = std::env::args().collect();
    let mut policy = String::from("workspace-write");
    let mut sandbox_policy_cwd: Option<PathBuf> = None;
    let mut codex_home: Option<PathBuf> = None;
    let mut command_cwd: Option<PathBuf> = None;
    let mut timeout_ms: Option<u64> = None;
    let mut elevated = false; // --elevated → full-admin backend (WFP network-block); needs UAC setup
    let mut command: Vec<String> = Vec::new();

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--policy" => policy = take(&args, &mut i)?,
            "--sandbox-policy-cwd" => sandbox_policy_cwd = Some(PathBuf::from(take(&args, &mut i)?)),
            "--command-cwd" => command_cwd = Some(PathBuf::from(take(&args, &mut i)?)),
            "--codex-home" => codex_home = Some(PathBuf::from(take(&args, &mut i)?)),
            "--timeout-ms" => timeout_ms = Some(take(&args, &mut i)?.parse().context("--timeout-ms")?),
            "--elevated" => {
                elevated = true;
                i += 1;
            }
            "--" => {
                command = args[i + 1..].to_vec();
                break;
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    if command.is_empty() {
        bail!("no command provided (expected `... -- <command> [args...]`)");
    }
    let sandbox_policy_cwd = sandbox_policy_cwd.context("--sandbox-policy-cwd is required")?;
    let command_cwd = command_cwd.unwrap_or_else(|| sandbox_policy_cwd.clone());
    let codex_home = codex_home.unwrap_or_else(std::env::temp_dir);
    let env_map: HashMap<String, String> = std::env::vars().collect();

    // --elevated = full-admin backend: codex internally runs the one-time UAC setup (sandbox user +
    // WFP network filters + ACLs) then spawns the command via codex-command-runner over named-pipe IPC.
    // Default = non-admin RestrictedToken (FS-write only). Both reuse codex Rust; we never port it.
    let result = if elevated {
        run_windows_sandbox_capture_elevated(ElevatedSandboxCaptureRequest {
            policy_json_or_preset: &policy,
            sandbox_policy_cwd: &sandbox_policy_cwd,
            codex_home: &codex_home,
            command,
            cwd: &command_cwd,
            env_map,
            timeout_ms,
            use_private_desktop: false,
            proxy_enforced: false,
            read_roots_override: None,
            read_roots_include_platform_defaults: true,
            write_roots_override: None,
            deny_read_paths_override: &[],
            deny_write_paths_override: &[],
        })?
    } else {
        run_windows_sandbox_capture(
            &policy,
            &sandbox_policy_cwd,
            &codex_home,
            command,
            &command_cwd,
            env_map,
            timeout_ms,
            false,
        )?
    };

    std::io::stdout().write_all(&result.stdout).ok();
    std::io::stderr().write_all(&result.stderr).ok();
    if result.timed_out {
        eprintln!("vdg-windows-restricted-spawn: command timed out");
    }
    std::process::exit(result.exit_code);
}

// Consume the value following the flag at args[i], advancing i past both.
#[cfg(target_os = "windows")]
fn take(args: &[String], i: &mut usize) -> anyhow::Result<String> {
    use anyhow::Context;
    let v = args
        .get(*i + 1)
        .cloned()
        .with_context(|| format!("missing value after {}", args[*i]))?;
    *i += 2;
    Ok(v)
}

#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("vdg-windows-restricted-spawn is Windows-only");
}
