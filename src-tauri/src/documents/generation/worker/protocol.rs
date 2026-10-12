use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use super::super::{input::DocumentInput, DocumentFormat};

pub(super) const VERSION: u16 = 2;
pub(super) const ARGUMENT: &str = "--knovum-document-worker-v2";
pub(super) const MAX_REQUEST_BYTES: u32 = 16 * 1024 * 1024;
pub(super) const MAX_HEADER_BYTES: u32 = 16 * 1024;
pub(super) const MAX_OUTPUT_BYTES: u32 = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub version: u16,
    pub format: DocumentFormat,
    pub source: DocumentInput,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Response {
    pub version: u16,
    pub outcome: Outcome,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Outcome {
    Rendered { length: u32 },
    Failed { message: String },
}

pub(super) fn encode_request(
    format: DocumentFormat,
    source: DocumentInput,
) -> Result<Vec<u8>, String> {
    source.validate()?;
    let payload = serde_json::to_vec(&Request {
        version: VERSION,
        format,
        source,
    })
    .map_err(|_| "Document worker request cannot be encoded")?;
    if payload.len() > MAX_REQUEST_BYTES as usize {
        return Err("Document worker request exceeds the frame limit".into());
    }
    let mut bytes = Vec::with_capacity(payload.len() + 4);
    write_frame(&mut bytes, &payload)?;
    Ok(bytes)
}

pub(super) fn read_request(reader: &mut impl Read) -> Result<Request, String> {
    let bytes = read_frame(reader, MAX_REQUEST_BYTES)?;
    let request: Request =
        serde_json::from_slice(&bytes).map_err(|_| "Document worker request is malformed")?;
    if request.version != VERSION {
        return Err("Document worker protocol version differs".into());
    }
    let mut trailing = [0];
    if reader
        .read(&mut trailing)
        .map_err(|error| error.to_string())?
        != 0
    {
        return Err("Document worker request has trailing bytes".into());
    }
    Ok(request)
}

pub(super) fn read_frame(reader: &mut impl Read, limit: u32) -> Result<Vec<u8>, String> {
    let mut length = [0; 4];
    reader
        .read_exact(&mut length)
        .map_err(|_| "Document worker frame is truncated")?;
    let length = u32::from_be_bytes(length);
    if length == 0 || length > limit {
        return Err("Document worker frame length is invalid".into());
    }
    let mut bytes = vec![0; length as usize];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| "Document worker frame is truncated")?;
    Ok(bytes)
}

pub(super) fn write_frame(writer: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let length =
        u32::try_from(bytes.len()).map_err(|_| "Document worker frame exceeds 32-bit length")?;
    writer
        .write_all(&length.to_be_bytes())
        .and_then(|_| writer.write_all(bytes))
        .map_err(|error| error.to_string())
}
