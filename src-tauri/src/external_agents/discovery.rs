use std::{path::PathBuf, time::Duration};

use crate::ai::contracts::AgentInstallation;
use crate::app_error::{AppError, AppResult};

const SHELL_PATH_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn system_path() -> AppResult<String> {
    #[cfg(target_os = "macos")]
    return login_shell_path().await;

    #[cfg(not(target_os = "macos"))]
    std::env::var("PATH").map_err(|_| AppError::internal("PATH is not available to Knovum"))
}

#[cfg(target_os = "macos")]
async fn login_shell_path() -> AppResult<String> {
    let shell = std::env::var("SHELL")
        .map_err(|_| AppError::internal("The macOS login shell is not available to Knovum"))?;
    let mut command = tokio::process::Command::new(shell);
    command
        .args(["-l", "-c", "echo \"__KNOVUM_PATH__:$PATH:__KNOVUM_PATH__\""])
        .kill_on_drop(true);
    let output = tokio::time::timeout(SHELL_PATH_TIMEOUT, command.output())
        .await
        .map_err(|_| AppError::internal("Reading the macOS login-shell PATH timed out"))?
        .map_err(|error| AppError::internal(error.to_string()))?;
    if !output.status.success() {
        return Err(AppError::internal(format!(
            "The macOS login shell exited with {} while reading PATH",
            output.status
        )));
    }
    extract_delimited(&String::from_utf8_lossy(&output.stdout), "__KNOVUM_PATH__:")
        .filter(|path| !path.is_empty())
        .ok_or_else(|| AppError::internal("The macOS login shell returned an invalid PATH"))
}

fn extract_delimited(text: &str, prefix: &str) -> Option<String> {
    let start = text.find(prefix)? + prefix.len();
    let rest = &text[start..];
    let end = rest.find(":__KNOVUM_PATH__")?;
    Some(rest[..end].trim().to_owned())
}

#[cfg(windows)]
const EXE_EXTENSIONS: &[&str] = &["", ".exe", ".cmd", ".bat"];
#[cfg(not(windows))]
const EXE_EXTENSIONS: &[&str] = &[""];

pub fn find_executable(name: &str, path: &str) -> Option<PathBuf> {
    for directory in std::env::split_paths(path) {
        for extension in EXE_EXTENSIONS {
            let candidate = directory.join(format!("{name}{extension}"));
            if candidate.is_file() && is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &std::path::Path) -> bool {
    path.is_file()
}

pub async fn detect_agent_installations() -> AppResult<Vec<AgentInstallation>> {
    let path = system_path().await?;
    tokio::task::spawn_blocking(move || -> AppResult<Vec<AgentInstallation>> {
        #[cfg(any(target_os = "macos", windows))]
        let installed_clients = super::installed_clients::detect()?;
        #[cfg(not(any(target_os = "macos", windows)))]
        let installed_clients = std::collections::HashSet::new();
        Ok(super::registry::descriptors()
            .into_iter()
            .map(|descriptor| {
                let executable_path = descriptor
                    .executable_names
                    .iter()
                    .find_map(|name| find_executable(name, &path))
                    .map(|path| path.to_string_lossy().into_owned());
                AgentInstallation {
                    kind: descriptor.kind,
                    detected: executable_path.is_some()
                        || installed_clients.contains(&descriptor.kind),
                    executable_path,
                }
            })
            .collect())
    })
    .await
    .map_err(|error| AppError::internal(error.to_string()))?
}
