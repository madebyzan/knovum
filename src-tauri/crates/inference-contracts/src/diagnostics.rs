use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeDiagnosticEvent {
    RequestFailed,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeDiagnosticCode {
    InvalidOutput,
    RuntimeMismatch,
    ResourceExhausted,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub enum RuntimeDiagnosticMethod {
    #[serde(rename = "runtime.initialize")]
    Initialize,
    #[serde(rename = "runtime.health")]
    Health,
    #[serde(rename = "runtime.cancel")]
    Cancel,
    #[serde(rename = "runtime.ackChunk")]
    AckChunk,
    #[serde(rename = "runtime.shutdown")]
    Shutdown,
    #[serde(rename = "embedding.encode")]
    Embedding,
    #[serde(rename = "image.segment")]
    Image,
    #[serde(rename = "speech.transcribe")]
    Transcribe,
    #[serde(rename = "speech.align")]
    Align,
    #[serde(rename = "speech.diarize")]
    Diarize,
    #[serde(rename = "speech.synthesize")]
    Synthesize,
    #[serde(rename = "speech.separate")]
    Separate,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeDiagnosticFrame {
    #[schemars(length(max = 128))]
    pub file: String,
    #[schemars(length(max = 128))]
    pub function: String,
    pub line: u32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeDiagnostic {
    pub event: RuntimeDiagnosticEvent,
    pub code: RuntimeDiagnosticCode,
    pub method: RuntimeDiagnosticMethod,
    pub request_id: Option<String>,
    #[schemars(length(max = 128))]
    pub exception_type: String,
    #[schemars(length(max = 8))]
    pub frames: Vec<RuntimeDiagnosticFrame>,
}
