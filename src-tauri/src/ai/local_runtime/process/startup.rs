#[cfg(windows)]
use super::windows_job::WindowsJob;
use super::{KillRegistration, ProcessTree, Worker};
use crate::ai::contracts::runtime::{RuntimeInitializeParams, RuntimeSession, PROTOCOL_VERSION};
use crate::ai::local_runtime::{
    admission,
    protocol::{io_error, Protocol},
};
use crate::local_resources::runtimes::python::PythonRuntimePaths;
use std::{
    path::Path,
    process::Stdio,
    sync::OnceLock,
    time::{Duration, Instant},
};
use tokio::{process::Command, sync::oneshot};
use tokio_util::sync::CancellationToken;

impl Worker {
    pub(in crate::ai::local_runtime) async fn start(
        runtime: &PythonRuntimePaths,
        work_directory: &Path,
        model_path: &Path,
        params: RuntimeInitializeParams,
        deadline_ms: u64,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Self, String> {
        let startup = Self::start_inner(
            runtime,
            work_directory,
            model_path,
            params,
            deadline_ms,
            cancellation,
            exit,
        )
        .await;
        if startup.is_err() {
            let _ = tokio::fs::remove_dir_all(work_directory).await;
        }
        startup
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
        if crate::ai::local_runtime::runtime_profile::for_execution(
            params.capability,
            &params.device_profile,
        )? != runtime.profile
        {
            return Err(
                "runtime_mismatch: worker package does not own the execution profile".into(),
            );
        }
        let memory_reservation =
            admission::reserve_memory(params.capability, &params.device_profile)?;
        tokio::select! {
            biased;
            _ = exit.cancelled() => return Err("cancelled".into()),
            _ = cancellation.cancelled() => return Err("cancelled".into()),
            startup = tokio::time::timeout(crate::ai::local_runtime::deadline_duration(deadline_ms)?, async {
                crate::ai::local_runtime::prepare_inference_root(work_directory).await?;
                verify_runtime_for_spawn(runtime.clone()).await?;
                tokio::fs::create_dir(work_directory)
                    .await
                    .map_err(|error| format!("resource_exhausted: {error}"))
            }) => startup.map_err(|_| "deadline_exceeded".to_owned())??,
        }
        let bootstrap = crate::subprocess::python::isolated_bootstrap(
            "from knovum_inference.server import main; main()",
        );
        let mut command = Command::new(&runtime.executable);
        command
            .args(["-I", "-B", "-X", "utf8", "-c", &bootstrap])
            .arg(&runtime.worker_root)
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
        tokio::spawn(super::diagnostics::observe(diagnostics));
        let mut protocol = Protocol::new(input, output);
        // Once spawned, cancellation must settle the child before releasing its reservation.
        let initialization: Result<RuntimeSession, String> =
            match crate::ai::local_runtime::deadline_duration(deadline_ms) {
                Err(error) => Err(error),
                Ok(remaining) => tokio::select! {
                    biased;
                    _ = exit.cancelled() => Err("cancelled".into()),
                    _ = cancellation.cancelled() => Err("cancelled".into()),
                    startup = tokio::time::timeout(
                        remaining.min(Duration::from_secs(15)),
                        protocol.request(&params.session_id, "runtime.initialize", &params),
                    ) => startup
                        .map_err(|_| "deadline_exceeded: worker initialization timed out".to_owned())
                        .and_then(std::convert::identity),
                },
            };
        let startup = match initialization {
            Ok(startup) => startup,
            Err(error) => {
                process_tree.terminate();
                let _ = child.kill().await;
                let _ = child.wait().await;
                let _ = tokio::fs::remove_dir_all(work_directory).await;
                return Err(error);
            }
        };
        if startup.protocol_version != PROTOCOL_VERSION
            || startup.runtime_digest != params.runtime_digest
            || startup.session_id != params.session_id
            || startup.capability != params.capability
            || startup.capability_spec_id != params.capability_spec_id
            || startup.device_profile.device != params.device_profile.device
            || startup.device_profile.execution_profile_id
                != params.device_profile.execution_profile_id
            || startup.device_profile.cpu_threads != params.device_profile.cpu_threads
            || startup.capabilities.len() != 1
            || startup.capabilities[0] != params.capability.method()
            || startup.limits.max_message_bytes != 1_048_576
            || startup.limits.max_embedding_texts != 32
            || startup.limits.max_embedding_tokens != 2048
            || startup.limits.max_unconsumed_chunk_files != 8
            || startup.limits.max_unconsumed_chunk_bytes != 16 * 1024 * 1024
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
            _runtime_lease: runtime.lease.clone(),
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
}

static VERIFY_GATE: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

pub(in crate::ai::local_runtime) async fn verify_runtime_for_spawn(
    runtime: PythonRuntimePaths,
) -> Result<(), String> {
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let gate = VERIFY_GATE.get_or_init(|| tokio::sync::Mutex::new(()));
        let _guard = gate.lock().await;
        let startup = tokio::task::spawn_blocking(move || {
            crate::local_resources::runtimes::python::verify_before_spawn(&runtime)
        })
        .await
        .map_err(|error| format!("python-runtime-invalid: verification task failed: {error}"))
        .and_then(std::convert::identity);
        let _ = sender.send(startup);
    });
    receiver
        .await
        .map_err(|_| "python-runtime-invalid: verification stopped".to_string())?
}
