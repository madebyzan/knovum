use std::process::Stdio;

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
};
use tokio_util::sync::CancellationToken;

use super::protocol;
use crate::{
    foundation::app_error::{AppError, AppResult},
    foundation::subprocess::process_tree::ProcessTree,
};

pub(super) async fn run(
    mut command: Command,
    request: Vec<u8>,
    cancellation: &CancellationToken,
) -> AppResult<Vec<u8>> {
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("Document rendering was cancelled"));
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .env_clear();
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
    let job =
        crate::foundation::subprocess::windows_job::WindowsJob::new().map_err(worker_error)?;
    let mut child = command.spawn().map_err(worker_error)?;
    #[cfg(windows)]
    if let Err(error) = job.assign(&child) {
        stop_child(&mut child).await?;
        return Err(worker_error(error));
    }
    let mut tree = match ProcessTree::new(
        &child,
        #[cfg(windows)]
        job,
    ) {
        Ok(tree) => tree,
        Err(error) => {
            stop_child(&mut child).await?;
            return Err(worker_error(error));
        }
    };
    let outcome = {
        let exchange = async {
            let mut input = child
                .stdin
                .take()
                .ok_or_else(|| worker_error("Document worker stdin unavailable"))?;
            let output = child
                .stdout
                .take()
                .ok_or_else(|| worker_error("Document worker stdout unavailable"))?;
            let write = async {
                input.write_all(&request).await.map_err(worker_error)?;
                input.shutdown().await.map_err(worker_error)?;
                drop(input);
                Ok::<_, AppError>(())
            };
            let wait = async { child.wait().await.map_err(worker_error) };
            let (_, bytes, status) = tokio::try_join!(write, read_response(output), wait)?;
            if !status.success() {
                return Err(worker_error("Document worker exited unsuccessfully"));
            }
            bytes
        };
        tokio::pin!(exchange);
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(AppError::cancelled("Document rendering was cancelled")),
            rendering = &mut exchange => rendering,
        }
    };
    // Even a protocol error must reap the process before the caller releases its Run permit.
    tree.terminate();
    if let Err(exit_error) = stop_child(&mut child).await {
        return Err(match outcome {
            Err(primary) => AppError {
                code: primary.code,
                message: format!("{}; {exit_error}", primary.message),
            },
            Ok(_) => exit_error,
        });
    }
    outcome
}

async fn stop_child(child: &mut Child) -> AppResult<()> {
    if child.try_wait().map_err(worker_error)?.is_none() {
        // start_kill leaves reaping to wait; a process that already exited still gets waited.
        let kill = child.start_kill();
        child.wait().await.map_err(worker_error)?;
        kill.map_err(worker_error)?;
    }
    Ok(())
}

async fn read_response(mut stream: impl AsyncRead + Unpin) -> AppResult<AppResult<Vec<u8>>> {
    let mut length = [0; 4];
    stream.read_exact(&mut length).await.map_err(worker_error)?;
    let length = u32::from_be_bytes(length);
    if length == 0 || length > protocol::MAX_HEADER_BYTES {
        return Err(worker_error(
            "Document worker response frame length is invalid",
        ));
    }
    let mut header = vec![0; length as usize];
    stream.read_exact(&mut header).await.map_err(worker_error)?;
    let response: protocol::Response = serde_json::from_slice(&header).map_err(worker_error)?;
    if response.version != protocol::VERSION {
        return Err(worker_error("Document worker response version differs"));
    }
    let rendering = match response.outcome {
        protocol::Outcome::Rendered { length } if length <= protocol::MAX_OUTPUT_BYTES => {
            let mut bytes = vec![0; length as usize];
            stream.read_exact(&mut bytes).await.map_err(worker_error)?;
            Ok(bytes)
        }
        protocol::Outcome::Rendered { .. } => {
            return Err(worker_error("Document worker output exceeds 64 MiB"))
        }
        protocol::Outcome::Failed { message } => {
            Err(AppError::new("document_render_failed", message))
        }
    };
    let mut trailing = [0];
    if stream.read(&mut trailing).await.map_err(worker_error)? != 0 {
        return Err(worker_error("Document worker response has trailing bytes"));
    }
    Ok(rendering)
}

fn worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "document_worker_failed",
        format!("Document worker failed: {error}"),
    )
}

#[cfg(test)]
#[path = "tests/response.rs"]
mod tests;
