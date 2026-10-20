use super::package::WorkerPackage;
#[cfg(windows)]
use super::windows_job::WindowsJob;
use super::{KillRegistration, ProcessTree, Worker, WorkerSpecification};
use crate::ai::contracts::runtime::{RuntimeInitializeParams, RuntimeSession, PROTOCOL_VERSION};
use crate::ai::local_runtime::{
    admission,
    protocol::{io_error, Protocol},
};
use crate::local_resources::runtimes::inference::InferenceRuntimePaths;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

impl Worker {
    /// Starts the one worker a capability asked for.
    ///
    /// A session identity, its private work directory under `inference_root` and its cold-start
    /// budget are properties of starting a worker rather than decisions a capability makes, so
    /// every capability reaches a running worker through this one path.
    pub(in crate::ai::local_runtime) async fn start(
        runtime: &InferenceRuntimePaths,
        inference_root: &Path,
        specification: WorkerSpecification<'_>,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Self, String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let work_directory = inference_root.join(&session_id);
        let startup = Self::start_inner(
            WorkerPackage::from(runtime),
            &work_directory,
            session_id,
            specification,
            cancellation,
            exit,
        )
        .await;
        if startup.is_err() {
            let _ = tokio::fs::remove_dir_all(&work_directory).await;
        }
        startup
    }

    async fn start_inner(
        runtime: WorkerPackage,
        work_directory: &Path,
        session_id: String,
        specification: WorkerSpecification<'_>,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Self, String> {
        // The host's verification and the worker's initialization read disjoint bytes, so each is
        // bounded by its own phase rather than by one value that describes neither. A cold start is
        // never bounded by the budget of the request that happened to need this worker:
        // verification and weight loading are shared costs.
        let verification = Duration::from_millis(runtime.spawn_verification_budget_ms());
        let model_path = specification.model_path;
        let model_bytes = tokio::select! {
            biased;
            _ = exit.cancelled() => return Err("cancelled".into()),
            _ = cancellation.cancelled() => return Err("cancelled".into()),
            startup = tokio::time::timeout(verification, async {
                runtime.verify().await?;
                crate::ai::local_runtime::prepare_inference_root(work_directory).await?;
                tokio::fs::create_dir(work_directory)
                    .await
                    .map_err(|error| format!("resource_exhausted: {error}"))?;
                snapshot_bytes(model_path).await
            }) => startup.map_err(|_| "deadline_exceeded: worker runtime verification timed out".to_owned())??,
        };
        let params = RuntimeInitializeParams {
            runtime_manifest_path: runtime.manifest_path().to_string_lossy().into_owned(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: runtime.digest().to_owned(),
            session_id,
            capability: specification.capability,
            capability_spec_id: specification.capability_spec_id,
            device_profile: specification.device_profile,
            work_directory: work_directory.to_string_lossy().into_owned(),
            model_paths: specification.model_paths,
            // Derived where both the verified runtime and the snapshot it loads are known.
            budget_ms: runtime.initialization_budget_ms(model_bytes),
        };
        let initialization = Duration::from_millis(params.budget_ms);
        let memory_reservation =
            admission::reserve_memory(params.capability, &params.device_profile)?;
        let mut command = runtime.command(params.device_profile.cpu_threads);
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
        let initialized: Result<RuntimeSession, String> = tokio::select! {
            biased;
            _ = exit.cancelled() => Err("cancelled".into()),
            _ = cancellation.cancelled() => Err("cancelled".into()),
            startup = tokio::time::timeout(
                initialization,
                protocol.request(&params.session_id, "runtime.initialize", &params),
            ) => startup
                .map_err(|_| "deadline_exceeded: worker initialization timed out".to_owned())
                .and_then(std::convert::identity),
        };
        let startup = match initialized {
            Ok(startup) => startup,
            Err(error) => {
                settle(child, process_tree).await;
                return Err(error);
            }
        };
        if !startup.matches(&params) {
            settle(child, process_tree).await;
            return Err("runtime_mismatch: worker identity or capabilities differ".into());
        }
        Ok(Self {
            child,
            process_tree,
            memory_reservation: Some(memory_reservation),
            _kill_registration: kill_registration,
            protocol,
            session_id: params.session_id,
            runtime_digest: runtime.digest().to_owned(),
            model_path: model_path.to_owned(),
            capability: params.capability,
            capability_spec_id: params.capability_spec_id,
            device_profile: params.device_profile,
            work_directory: work_directory.to_owned(),
            last_used: Instant::now(),
        })
    }
}

/// Settles a worker that never became usable: the process group dies before its memory
/// reservation is released, and [`Worker::start`] removes the work directory it created.
async fn settle(mut child: tokio::process::Child, mut process_tree: ProcessTree) {
    process_tree.terminate();
    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// Measures the snapshot this worker is about to load, so its initialization budget accounts for
/// the weights as well as the libraries.
async fn snapshot_bytes(model_path: &Path) -> Result<u64, String> {
    let path: PathBuf = model_path.to_owned();
    crate::foundation::blocking::run(move || {
        crate::local_resources::filesystem::installed_bytes(&path)
    })
    .await
}
