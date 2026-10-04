use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::sync::Arc;

use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    sync::oneshot,
};
use tokio_util::sync::CancellationToken;

use super::protocol::{io_error, Protocol};
use crate::{
    ai::contracts::runtime::{
        RuntimeCapability, RuntimeHealthParams, RuntimeHealthResult, RuntimeInitializeParams,
        RuntimeInitializeResult, PROTOCOL_VERSION,
    },
    local_resources::tools::PythonRuntimePaths,
};

#[cfg(windows)]
#[path = "windows_job.rs"]
mod windows_job;
#[cfg(windows)]
use windows_job::WindowsJob;

pub(super) struct Worker {
    pub(super) child: Child,
    process_tree: ProcessTree,
    memory_reservation: Option<super::admission::MemoryReservation>,
    _kill_registration: KillRegistration,
    pub(super) protocol: Protocol,
    pub(super) session_id: String,
    pub(super) runtime_digest: String,
    pub(super) model_path: PathBuf,
    pub(super) capability: RuntimeCapability,
    pub(super) capability_spec_id: String,
    pub(super) device_profile: crate::ai::contracts::runtime::RuntimeDeviceProfile,
    work_directory: PathBuf,
    pub(super) last_used: Instant,
}

impl Worker {
    pub(super) fn work_directory(&self) -> &Path {
        &self.work_directory
    }

    pub(super) fn matches(
        &self,
        runtime_digest: &str,
        model_path: &Path,
        capability: RuntimeCapability,
        capability_spec_id: &str,
        device_profile: &crate::ai::contracts::runtime::RuntimeDeviceProfile,
    ) -> bool {
        self.runtime_digest == runtime_digest
            && self.model_path == model_path
            && self.capability == capability
            && self.capability_spec_id == capability_spec_id
            && self.device_profile == *device_profile
    }

    pub(super) async fn resident_bytes(&mut self) -> Result<u64, String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            self.protocol.request::<_, RuntimeHealthResult>(
                &request_id,
                "runtime.health",
                &RuntimeHealthParams {
                    session_id: self.session_id.clone(),
                },
            ),
        )
        .await
        .map_err(|_| "deadline_exceeded: runtime health timed out".to_owned())??;
        if result.session_id != self.session_id || result.current_request_id.is_some() {
            return Err("invalid_output: runtime health identity or worker state differs".into());
        }
        if result.resident_bytes == 0 {
            return Err("invalid_output: runtime health reported zero resident memory".into());
        }
        Ok(result.resident_bytes)
    }

    pub(super) async fn start(
        runtime: &PythonRuntimePaths,
        work_directory: &Path,
        model_path: &Path,
        params: RuntimeInitializeParams,
        deadline_ms: u64,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Self, String> {
        let result = Self::start_inner(
            runtime,
            work_directory,
            model_path,
            params,
            deadline_ms,
            cancellation,
            exit,
        )
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_dir_all(work_directory).await;
        }
        result
    }

    async fn start_inner(
        runtime: &PythonRuntimePaths,
        work_directory: &Path,
        model_path: &Path,
        params: RuntimeInitializeParams,
        deadline_ms: u64,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Self, String> {
        let memory_reservation =
            super::admission::reserve_memory(params.capability, &params.device_profile)?;
        tokio::select! {
            biased;
            _ = exit.cancelled() => return Err("cancelled".into()),
            _ = cancellation.cancelled() => return Err("cancelled".into()),
            result = tokio::time::timeout(super::deadline_duration(deadline_ms)?, async {
                super::prepare_inference_root(work_directory).await?;
                verify_runtime_for_spawn(runtime.clone()).await?;
                tokio::fs::create_dir(work_directory)
                    .await
                    .map_err(|error| format!("resource_exhausted: {error}"))
            }) => result.map_err(|_| "deadline_exceeded".to_owned())??,
        }
        let bootstrap = concat!(
            "import sys; ",
            "sys.path.insert(0, sys.argv[1]); ",
            "from knovum_inference.server import main; main()"
        );
        let mut command = Command::new(&runtime.executable);
        command
            .args(["-I", "-B", "-X", "utf8", "-c", bootstrap])
            .arg(&runtime.package_root)
            .current_dir(work_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env_remove("PYTHONSTARTUP")
            .env_remove("VOXCPM_MPS_DTYPE")
            .env_remove("HF_TOKEN")
            .env_remove("HUGGING_FACE_HUB_TOKEN")
            .env("HF_HUB_OFFLINE", "1")
            .env("TRANSFORMERS_OFFLINE", "1")
            .env("PYANNOTE_METRICS_ENABLED", "0")
            .env("HF_HUB_DISABLE_TELEMETRY", "1")
            .env("PYTORCH_ENABLE_MPS_FALLBACK", "0")
            .env(
                "OMP_NUM_THREADS",
                params.device_profile.cpu_threads.to_string(),
            )
            .env(
                "MKL_NUM_THREADS",
                params.device_profile.cpu_threads.to_string(),
            )
            .env(
                "OPENBLAS_NUM_THREADS",
                params.device_profile.cpu_threads.to_string(),
            )
            .env(
                "NUMEXPR_NUM_THREADS",
                params.device_profile.cpu_threads.to_string(),
            )
            .env(
                "VECLIB_MAXIMUM_THREADS",
                params.device_profile.cpu_threads.to_string(),
            )
            .kill_on_drop(true);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        #[cfg(windows)]
        let job = WindowsJob::new()?;
        let mut child = command.spawn().map_err(io_error)?;
        #[cfg(windows)]
        if let Err(error) = job.assign(&child) {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(error);
        }
        let mut process_tree = ProcessTree::new(
            &child,
            #[cfg(windows)]
            job,
        )?;
        let kill_registration = KillRegistration::new(&params.session_id, process_tree.handle());
        let input = child
            .stdin
            .take()
            .ok_or("worker_crashed: worker stdin unavailable")?;
        let output = child
            .stdout
            .take()
            .ok_or("worker_crashed: worker stdout unavailable")?;
        let diagnostics = child
            .stderr
            .take()
            .ok_or("worker_crashed: worker stderr unavailable")?;
        tokio::spawn(async move {
            let mut diagnostics = diagnostics;
            let mut chunk = [0_u8; 4096];
            let mut line = Vec::new();
            let mut oversized = false;
            while let Ok(size) = diagnostics.read(&mut chunk).await {
                if size == 0 {
                    break;
                }
                for byte in &chunk[..size] {
                    if *byte == b'\n' {
                        if !oversized {
                            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&line) {
                                if let Some(code) =
                                    value.get("code").and_then(serde_json::Value::as_str)
                                {
                                    log::warn!("local inference diagnostic code={code}");
                                }
                            }
                        }
                        line.clear();
                        oversized = false;
                    } else if !oversized {
                        if line.len() < 16 * 1024 {
                            line.push(*byte);
                        } else {
                            line.clear();
                            oversized = true;
                        }
                    }
                }
            }
        });
        let mut protocol = Protocol::new(input, output);
        // Once spawned, cancellation must settle the child before releasing its reservation.
        let initialization: Result<RuntimeInitializeResult, String> = match super::deadline_duration(
            deadline_ms,
        ) {
            Err(error) => Err(error),
            Ok(remaining) => tokio::select! {
                biased;
                _ = exit.cancelled() => Err("cancelled".into()),
                _ = cancellation.cancelled() => Err("cancelled".into()),
                result = tokio::time::timeout(
                    remaining.min(Duration::from_secs(15)),
                    protocol.request(&params.session_id, "runtime.initialize", &params),
                ) => result
                    .map_err(|_| "deadline_exceeded: worker initialization timed out".to_owned())
                    .and_then(std::convert::identity),
            },
        };
        let result = match initialization {
            Ok(result) => result,
            Err(error) => {
                process_tree.terminate();
                let _ = child.kill().await;
                let _ = child.wait().await;
                let _ = tokio::fs::remove_dir_all(work_directory).await;
                return Err(error);
            }
        };
        if result.protocol_version != PROTOCOL_VERSION
            || result.runtime_digest != params.runtime_digest
            || result.session_id != params.session_id
            || result.capability != params.capability
            || result.capability_spec_id != params.capability_spec_id
            || result.device_profile.device != params.device_profile.device
            || result.device_profile.execution_profile_id
                != params.device_profile.execution_profile_id
            || result.device_profile.cpu_threads != params.device_profile.cpu_threads
            || result.capabilities.len() != 1
            || result.capabilities[0] != params.capability.method()
            || result.limits.max_message_bytes != 1_048_576
            || result.limits.max_embedding_texts != 32
            || result.limits.max_embedding_tokens != 2048
            || result.limits.max_unconsumed_chunk_files != 8
            || result.limits.max_unconsumed_chunk_bytes != 16 * 1024 * 1024
        {
            process_tree.terminate();
            let _ = child.kill().await;
            let _ = child.wait().await;
            let _ = tokio::fs::remove_dir_all(work_directory).await;
            return Err("runtime_mismatch: worker identity or capabilities differ".into());
        }
        Ok(Self {
            child,
            process_tree,
            memory_reservation: Some(memory_reservation),
            _kill_registration: kill_registration,
            protocol,
            session_id: params.session_id,
            runtime_digest: runtime.digest.clone(),
            model_path: model_path.to_owned(),
            capability: params.capability,
            capability_spec_id: params.capability_spec_id,
            device_profile: params.device_profile,
            work_directory: work_directory.to_owned(),
            last_used: Instant::now(),
        })
    }

    pub(super) async fn terminate(&mut self) {
        self.process_tree.terminate();
        self._kill_registration.clear();
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        let _ = tokio::fs::remove_dir_all(&self.work_directory).await;
        if let Some(mut reservation) = self.memory_reservation.take() {
            reservation.release();
        }
    }
}

static VERIFY_GATE: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn verify_runtime_for_spawn(runtime: PythonRuntimePaths) -> Result<(), String> {
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let gate = VERIFY_GATE.get_or_init(|| tokio::sync::Mutex::new(()));
        let _guard = gate.lock().await;
        let result = tokio::task::spawn_blocking(move || {
            crate::local_resources::tools::verify_before_spawn(&runtime)
        })
        .await
        .map_err(|error| format!("python-runtime-invalid: verification task failed: {error}"))
        .and_then(std::convert::identity);
        let _ = sender.send(result);
    });
    receiver
        .await
        .map_err(|_| "python-runtime-invalid: verification stopped".to_string())?
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.process_tree.terminate();
        self._kill_registration.clear();
    }
}

struct ProcessTree(Option<KillHandle>);

#[derive(Clone)]
enum KillHandle {
    #[cfg(unix)]
    ProcessGroup(i32),
    #[cfg(windows)]
    Job(Arc<WindowsJob>),
}

static ACTIVE_KILL: OnceLock<Mutex<Option<(String, KillHandle)>>> = OnceLock::new();

struct KillRegistration {
    session_id: String,
}

impl KillRegistration {
    fn new(session_id: &str, handle: KillHandle) -> Self {
        let registry = ACTIVE_KILL.get_or_init(|| Mutex::new(None));
        if let Ok(mut active) = registry.lock() {
            *active = Some((session_id.to_owned(), handle));
        }
        Self {
            session_id: session_id.to_owned(),
        }
    }

    fn clear(&self) {
        if let Some(registry) = ACTIVE_KILL.get() {
            if let Ok(mut active) = registry.lock() {
                if active
                    .as_ref()
                    .is_some_and(|(id, _)| id == &self.session_id)
                {
                    *active = None;
                }
            }
        }
    }
}

impl Drop for KillRegistration {
    fn drop(&mut self) {
        self.clear();
    }
}

pub(super) fn force_terminate_active() {
    if let Some(registry) = ACTIVE_KILL.get() {
        if let Ok(active) = registry.lock() {
            if let Some((_, handle)) = active.as_ref() {
                handle.terminate();
            }
        }
    }
}

impl KillHandle {
    fn terminate(&self) {
        #[cfg(unix)]
        {
            let Self::ProcessGroup(group) = self;
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            let _ = unsafe { kill(-group, 9) };
        }
        #[cfg(windows)]
        {
            let Self::Job(job) = self;
            job.terminate();
        }
    }
}

impl ProcessTree {
    fn new(child: &Child, #[cfg(windows)] job: WindowsJob) -> Result<Self, String> {
        #[cfg(unix)]
        let process_group = i32::try_from(
            child
                .id()
                .ok_or("worker_crashed: worker process id unavailable")?,
        )
        .map_err(|_| "worker_crashed: worker process id exceeds platform range")?;
        Ok(Self(Some({
            #[cfg(unix)]
            {
                KillHandle::ProcessGroup(process_group)
            }
            #[cfg(windows)]
            {
                KillHandle::Job(Arc::new(job))
            }
        })))
    }

    fn handle(&self) -> KillHandle {
        self.0
            .as_ref()
            .expect("process tree has a kill handle")
            .clone()
    }

    fn terminate(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.terminate();
        }
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        self.terminate();
    }
}
