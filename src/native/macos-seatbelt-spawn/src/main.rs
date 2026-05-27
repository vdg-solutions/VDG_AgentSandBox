// VDG #472-E E-3 — runs a command under codex's macOS seatbelt sandbox. Depends on the vendored codex
// crates via their PUBLIC API (codex_sandboxing::seatbelt::create_seatbelt_command_args +
// PermissionProfile::to_runtime_permissions); it does NOT modify any codex source. FS-write is confined
// to the workspace and network is DENIED (both compiled into the seatbelt policy by codex). A C# agent
// spawns this bin like codex-linux-sandbox, passing the SAME --permission-profile JSON.
//
// CLI: --sandbox-policy-cwd <dir> [--command-cwd <dir>] --permission-profile <json> -- <command> [args...]

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;

    use anyhow::{bail, Context};
    use codex_protocol::models::PermissionProfile;
    use codex_sandboxing::seatbelt::{
        create_seatbelt_command_args, CreateSeatbeltCommandArgsParams,
        MACOS_PATH_TO_SEATBELT_EXECUTABLE,
    };

    let args: Vec<String> = std::env::args().collect();
    let mut sandbox_policy_cwd: Option<PathBuf> = None;
    let mut command_cwd: Option<PathBuf> = None;
    let mut profile_json: Option<String> = None;
    let mut command: Vec<String> = Vec::new();

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--sandbox-policy-cwd" => sandbox_policy_cwd = Some(PathBuf::from(take(&args, &mut i)?)),
            "--command-cwd" => command_cwd = Some(PathBuf::from(take(&args, &mut i)?)),
            "--permission-profile" => profile_json = Some(take(&args, &mut i)?),
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
    let profile_json = profile_json.context("--permission-profile is required")?;

    // Same PermissionProfile JSON the agent feeds codex-linux-sandbox; to_runtime_permissions() yields
    // the (filesystem, network) policies codex compiles into the seatbelt (.sb) profile.
    let profile: PermissionProfile =
        serde_json::from_str(&profile_json).context("invalid permission profile JSON")?;
    let (file_system_sandbox_policy, network_sandbox_policy) = profile.to_runtime_permissions();

    let mut sandbox_args = create_seatbelt_command_args(CreateSeatbeltCommandArgsParams {
        command,
        file_system_sandbox_policy: &file_system_sandbox_policy,
        network_sandbox_policy,
        sandbox_policy_cwd: &sandbox_policy_cwd,
        enforce_managed_network: false,
        network: None,
        extra_allow_unix_sockets: &[],
    });
    let mut full_command = Vec::with_capacity(1 + sandbox_args.len());
    full_command.push(MACOS_PATH_TO_SEATBELT_EXECUTABLE.to_string());
    full_command.append(&mut sandbox_args);

    // exec sandbox-exec (mirrors codex-linux-sandbox exec'ing bwrap): the command's stdio + exit code
    // flow straight to the agent that spawned us. current_dir = command_cwd so the inner command runs
    // where it expects (seatbelt runs argv in the process cwd).
    let err = Command::new(&full_command[0])
        .args(&full_command[1..])
        .current_dir(&command_cwd)
        .exec();
    bail!("failed to exec {}: {err}", MACOS_PATH_TO_SEATBELT_EXECUTABLE);
}

// Consume the value following the flag at args[i], advancing i past both.
#[cfg(target_os = "macos")]
fn take(args: &[String], i: &mut usize) -> anyhow::Result<String> {
    use anyhow::Context;
    let v = args
        .get(*i + 1)
        .cloned()
        .with_context(|| format!("missing value after {}", args[*i]))?;
    *i += 2;
    Ok(v)
}

#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("vdg-macos-seatbelt-spawn is macOS-only");
}
