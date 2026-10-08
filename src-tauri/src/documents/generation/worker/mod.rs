mod process;
mod protocol;

use std::io::{Read, Write};

use tokio_util::sync::CancellationToken;

use super::{document::DocumentSource, DocumentFormat};
use crate::app_error::{AppError, AppResult};

pub(crate) async fn render(
    format: DocumentFormat,
    source: DocumentSource,
    cancellation: &CancellationToken,
) -> AppResult<Vec<u8>> {
    let request = protocol::encode_request(format, source).map_err(AppError::invalid)?;
    let executable = std::env::current_exe().map_err(|_| {
        AppError::new(
            "document_worker_failed",
            "Application executable is unavailable",
        )
    })?;
    let mut command = tokio::process::Command::new(executable);
    command.arg(protocol::ARGUMENT);
    process::run(command, request, cancellation).await
}

/// Consumes the private worker entry before any desktop initialization.
pub(crate) fn run_if_requested() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments
        .first()
        .is_none_or(|arg| arg != protocol::ARGUMENT)
    {
        return;
    }
    if arguments.len() != 1 {
        std::process::exit(2);
    }
    let rendering = serve(&mut std::io::stdin().lock(), &mut std::io::stdout().lock());
    std::process::exit(if rendering.is_ok() { 0 } else { 1 });
}

fn serve(reader: &mut impl Read, writer: &mut impl Write) -> Result<(), String> {
    let request = protocol::read_request(reader)?;
    let rendering = request.format.render(&request.source);
    let (outcome, bytes) = match rendering {
        Ok(bytes) if bytes.len() <= protocol::MAX_OUTPUT_BYTES as usize => (
            protocol::Outcome::Rendered {
                length: bytes.len() as u32,
            },
            bytes,
        ),
        Ok(_) => (
            protocol::Outcome::Failed {
                message: "Document output exceeds 64 MiB".into(),
            },
            vec![],
        ),
        Err(error) => (
            protocol::Outcome::Failed {
                message: error.chars().take(2048).collect(),
            },
            vec![],
        ),
    };
    let header = serde_json::to_vec(&protocol::Response {
        version: protocol::VERSION,
        outcome,
    })
    .map_err(|error| error.to_string())?;
    protocol::write_frame(writer, &header)?;
    writer
        .write_all(&bytes)
        .and_then(|_| writer.flush())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "tests/process.rs"]
mod process_tests;
#[cfg(test)]
#[path = "tests/protocol.rs"]
mod protocol_tests;
