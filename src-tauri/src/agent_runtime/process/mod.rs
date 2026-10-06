use std::{path::Path, process::Stdio, sync::Arc, time::Duration};

use tauri::Manager;
use tokio::{
    process::{Child, Command},
    task::JoinHandle,
};

use super::protocol::{
    self,
    control::{AgentInitialize, AgentReady},
};
use crate::local_resources::runtimes::python::{self, RuntimeProfile};
use crate::subprocess::process_tree::ProcessTree;

mod framing;
mod peer;
pub(crate) use peer::{Host, Peer};

pub(crate) struct Process {
    child: Child,
    tree: ProcessTree,
    listener: JoinHandle<Result<(), String>>,
    pub(crate) peer: Peer,
    _lease: Option<Arc<std::fs::File>>,
}

impl Process {
    pub(crate) async fn start(
        app: &tauri::AppHandle,
        checkpoint_path: &Path,
        host: Arc<dyn Host>,
    ) -> Result<Self, String> {
        let runtime = python::resolve(app, RuntimeProfile::Agent)?
            .ok_or("agent-runtime-runtime-not-prepared")?;
        let checked = runtime.clone();
        crate::blocking::run(move || python::verify_before_spawn(&checked)).await?;
        let work = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?
            .join("agent-runtime");
        tokio::fs::create_dir_all(&work)
            .await
            .map_err(|error| error.to_string())?;
        let bootstrap = "import sys;sys.path[:0]=[sys.argv[1],sys.argv[2]];import runpy;runpy.run_module('knovum_agent_runtime',run_name='__main__')";
        let mut command = Command::new(&runtime.executable);
        command
            .args(["-I", "-B", "-X", "utf8", "-c", bootstrap])
            .arg(&runtime.worker_root)
            .arg(&runtime.package_root)
            .current_dir(work)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .env_clear();
        // Explicit environment ownership prevents API-key, telemetry and Python
        // configuration inherited from the desktop's launcher entering this process.
        #[cfg(windows)]
        for key in ["SYSTEMROOT", "WINDIR", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        #[cfg(windows)]
        let job = crate::subprocess::windows_job::WindowsJob::new()?;
        let mut child = command
            .spawn()
            .map_err(|_| "agent-runtime-process-start-failed")?;
        #[cfg(windows)]
        job.assign(&child)?;
        let tree = ProcessTree::new(
            &child,
            #[cfg(windows)]
            job,
        )?;
        let input = child
            .stdin
            .take()
            .ok_or("agent-runtime-stdin-unavailable")?;
        let output = child
            .stdout
            .take()
            .ok_or("agent-runtime-stdout-unavailable")?;
        let diagnostics = child
            .stderr
            .take()
            .ok_or("agent-runtime-stderr-unavailable")?;
        // Provider/Pydantic errors may include credentials: discard stderr while
        // draining it to preserve pipe backpressure; report typed protocol failures.
        tokio::spawn(async move {
            let _ = tokio::io::copy(
                &mut tokio::io::BufReader::new(diagnostics),
                &mut tokio::io::sink(),
            )
            .await;
        });
        let peer = Peer::new(input);
        let receiver = peer.clone();
        let listener = tokio::spawn(async move {
            let outcome = receiver.listen(output, host).await;
            receiver.disconnect();
            outcome
        });
        let mut process = Self {
            child,
            tree,
            listener,
            peer,
            _lease: runtime.lease,
        };
        let ready = tokio::time::timeout(
            Duration::from_secs(20),
            process.peer.request::<AgentReady>(
                "agentRuntime.initialize",
                &AgentInitialize {
                    protocol_version: protocol::PROTOCOL_VERSION,
                    graph_version: protocol::GRAPH_VERSION.into(),
                    checkpoint_path: checkpoint_path
                        .to_str()
                        .ok_or("agent-runtime-checkpoint-path-invalid")?
                        .into(),
                },
            ),
        )
        .await
        .map_err(|_| "agent-runtime-initialize-timeout".to_owned())
        .and_then(std::convert::identity);
        match ready {
            Ok(ready)
                if ready.protocol_version == protocol::PROTOCOL_VERSION
                    && ready.graph_version == protocol::GRAPH_VERSION =>
            {
                Ok(process)
            }
            _ => {
                process.terminate().await;
                Err("agent-runtime-initialization-failed".into())
            }
        }
    }

    pub(crate) async fn terminate(&mut self) {
        self.tree.terminate();
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        if tokio::time::timeout(Duration::from_secs(5), &mut self.listener)
            .await
            .is_err()
        {
            self.listener.abort();
        }
        self.peer.disconnect();
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.tree.terminate();
        self.listener.abort();
        self.peer.disconnect();
    }
}

#[cfg(test)]
mod tests;
