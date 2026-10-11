pub mod diagnostics;
pub mod embedding_wire;
pub mod image_compute;
pub mod image_wire;
pub mod native_manifest;
pub mod runtime;
pub mod speech_wire;

#[cfg(test)]
#[path = "tests/protocol.rs"]
mod tests;
