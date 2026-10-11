use serde::Serialize;
use std::{fs, path::PathBuf};
#[cfg(not(debug_assertions))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};
#[cfg(any(test, not(debug_assertions)))]
mod inventory;
#[cfg(any(test, not(debug_assertions)))]
mod paths;
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(crate) struct AgentRuntimePaths {
    pub(crate) executable: PathBuf,
    pub(crate) arguments: Vec<&'static str>,
}
#[derive(Serialize, ts_rs::TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum AgentRuntimeStatus {
    Ready,
    Invalid { message: String },
}

pub(crate) fn resolve<R: Runtime>(app: &AppHandle<R>) -> Result<AgentRuntimePaths, String> {
    #[cfg(debug_assertions)]
    let runtime = {
        let _ = app;
        let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../python");
        let executable = project.join(if cfg!(windows) {
            ".venv/Scripts/python.exe"
        } else {
            ".venv/bin/python"
        });
        AgentRuntimePaths {
            executable,
            arguments: vec!["-I", "-B", "-X", "utf8", "-m", "knovum_agent_runtime"],
        }
    };
    #[cfg(not(debug_assertions))]
    let runtime = {
        let root = app
            .path()
            .resource_dir()
            .map_err(|e| e.to_string())?
            .join("resources/agent-runtime/knovum-agent");
        inventory::verify(
            &root,
            include_bytes!(concat!(env!("OUT_DIR"), "/agent-manifest.json")),
        )?;
        AgentRuntimePaths {
            executable: root.join(if cfg!(windows) {
                "knovum-agent.exe"
            } else {
                "knovum-agent"
            }),
            arguments: vec![],
        }
    };
    let metadata = fs::metadata(&runtime.executable)
        .map_err(|e| format!("agent-runtime-executable-missing: {e}"))?;
    if !metadata.is_file() {
        return Err("agent-runtime-executable-invalid".into());
    }
    Ok(runtime)
}
pub(crate) fn status<R: Runtime>(app: &AppHandle<R>) -> AgentRuntimeStatus {
    match resolve(app) {
        Ok(_) => AgentRuntimeStatus::Ready,
        Err(message) => AgentRuntimeStatus::Invalid { message },
    }
}
