//! How a command line becomes a process on this computer: which shell reads it, what confines it,
//! and what environment it sees.

use std::{ffi::OsString, path::Path, process::Stdio, time::Duration};

use tokio::process::Command;

use super::output::{self, Exit, Finished};
use crate::foundation::subprocess::process_tree::ProcessTree;

#[cfg(target_os = "macos")]
pub(super) const CONFINED: bool = true;
#[cfg(not(target_os = "macos"))]
pub(super) const CONFINED: bool = false;

#[cfg(target_os = "macos")]
pub(super) const SYSTEM: &str = "macOS";
#[cfg(target_os = "macos")]
pub(super) const SHELL: &str = "zsh";

#[cfg(windows)]
pub(super) const SYSTEM: &str = "Windows";
#[cfg(windows)]
pub(super) const SHELL: &str = "Windows PowerShell 5.1, not bash or cmd: write PowerShell — cmdlets such as Get-ChildItem, Get-Content and Select-String, `$env:NAME` for environment variables, and `;` between statements";

#[cfg(all(unix, not(target_os = "macos")))]
pub(super) const SYSTEM: &str = "Linux";
#[cfg(all(unix, not(target_os = "macos")))]
pub(super) const SHELL: &str = "sh";

/// The variables a command inherits. Everything else in Knovum's environment — provider keys,
/// tokens, the user's other secrets — stays out, as deepagents' `inherit_env=False` keeps it out.
#[cfg(unix)]
pub(super) const INHERITED: &[&str] = &[
    "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR", "TZ",
];
#[cfg(windows)]
pub(super) const INHERITED: &[&str] = &[
    "SystemRoot",
    "SystemDrive",
    "windir",
    "ComSpec",
    "PATHEXT",
    "TEMP",
    "TMP",
    "USERNAME",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "APPDATA",
    "LOCALAPPDATA",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "CommonProgramFiles",
    "PSModulePath",
    "PROCESSOR_ARCHITECTURE",
    "NUMBER_OF_PROCESSORS",
    "OS",
];

/// Runs `command_line` in `workspace` until it exits or `timeout` passes. Every process it started
/// ends with it — a background process too, which would otherwise hold the output open.
pub(super) async fn run(
    workspace: &Path,
    command_line: &str,
    timeout: Duration,
) -> Result<Finished, String> {
    std::fs::create_dir_all(workspace).map_err(|error| error.to_string())?;
    let mut command = shell(workspace, command_line)?;
    command
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env_clear()
        .envs(environment().await?);
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    #[cfg(windows)]
    let job = crate::foundation::subprocess::windows_job::WindowsJob::new()?;
    let mut child = command
        .spawn()
        .map_err(|error| format!("command-start-failed: {error}"))?;
    let mut tree = ProcessTree::new(
        &child,
        #[cfg(windows)]
        job,
    )?;
    let stdout = child.stdout.take().ok_or("command-output-unavailable")?;
    let stderr = child.stderr.take().ok_or("command-output-unavailable")?;
    let (exit, output) = tokio::join!(
        async {
            let exit = match tokio::time::timeout(timeout, child.wait()).await {
                Ok(Ok(status)) => Ok(status.code().map_or(Exit::Signal, Exit::Code)),
                Ok(Err(error)) => Err(error.to_string()),
                Err(_) => Ok(Exit::TimedOut),
            };
            tree.terminate();
            exit
        },
        output::capture(stdout, stderr),
    );
    Ok(Finished {
        output,
        exit: exit?,
    })
}

async fn environment() -> Result<Vec<(OsString, OsString)>, String> {
    let path = crate::external_agents::discovery::system_path()
        .await
        .map_err(|error| error.to_string())?;
    Ok(INHERITED
        .iter()
        .filter_map(|name| std::env::var_os(name).map(|value| (OsString::from(name), value)))
        .chain([(OsString::from("PATH"), OsString::from(path))])
        .collect())
}

/// Reads and runs anything, writes only the working directory, the temporary directories and the
/// terminal devices, and reaches no network. Of the rules that match, the last one decides.
#[cfg(target_os = "macos")]
const SEATBELT_PROFILE: &str = r##"(version 1)
(allow default)
(deny network*)
(deny file-write*)
(allow file-write*
    (subpath (param "WORKSPACE"))
    (subpath (param "TEMPORARY"))
    (subpath "/private/tmp")
    (literal "/dev/null")
    (literal "/dev/zero")
    (literal "/dev/tty")
    (literal "/dev/dtracehelper")
    (regex #"^/dev/fd/[0-9]+$"))
"##;

#[cfg(target_os = "macos")]
fn shell(workspace: &Path, command_line: &str) -> Result<Command, String> {
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .arg("-p")
        .arg(SEATBELT_PROFILE)
        .arg("-D")
        .arg(parameter("WORKSPACE", workspace)?)
        .arg("-D")
        .arg(parameter("TEMPORARY", &std::env::temp_dir())?)
        .args(["/bin/zsh", "-c", command_line]);
    Ok(command)
}

/// Seatbelt matches the paths a process really touches, so a directory is named by its real path
/// (`/var/folders/…` is `/private/var/folders/…`).
#[cfg(target_os = "macos")]
fn parameter(name: &str, directory: &Path) -> Result<String, String> {
    let real = std::fs::canonicalize(directory).map_err(|error| error.to_string())?;
    let real = real.to_str().ok_or("command-directory-not-unicode")?;
    Ok(format!("{name}={real}"))
}

/// PowerShell reads the command encoded, so no quoting stands between the Agent's command line and
/// the shell, and writes UTF-8, so non-ASCII output survives the pipe.
#[cfg(windows)]
fn shell(_workspace: &Path, command_line: &str) -> Result<Command, String> {
    use base64::Engine;

    let script =
        format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8\n{command_line}");
    let encoded: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
        ])
        .arg(base64::engine::general_purpose::STANDARD.encode(encoded));
    Ok(command)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn shell(_workspace: &Path, command_line: &str) -> Result<Command, String> {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", command_line]);
    Ok(command)
}
