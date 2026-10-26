pub mod chat;
pub use knovum_inference_contracts::diagnostics;
pub mod embedding;
pub mod embedding_wire;
pub use knovum_inference_contracts::execution_profile;
pub mod external_agent;
pub mod files;
pub use knovum_inference_contracts::image_compute;
pub mod image_wire;
pub mod media;
pub mod model_turn;
pub mod provider;
pub use knovum_inference_contracts::runtime;
pub use knovum_inference_contracts::speech_wire;

pub use external_agent::{
    ExternalAgentConnectionKind, ExternalAgentDescriptor, ExternalAgentHelp,
    ExternalAgentInstallation, ExternalAgentKind,
};

pub use chat::{ChatRequest, ChatResponse, StreamFrame, StreamFrameStream, TokenUsage};
pub use embedding::{EmbeddingCapability, EmbeddingResponse};
pub use files::{FileUploadCapability, UploadedFileReference};
pub use media::{
    GeneratedImage, GeneratedVideo, ImageGenerationCapability, ImageGenerationInput,
    ImageReference, ImageReferenceSupport, VideoGenerationCapability, VideoGenerationInput,
    VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
};
pub use model_turn::{
    ModelBinarySource, ModelContentPart, ModelFinishReason, ModelMessage, ModelRole,
    ModelToolChoice, ModelToolDefinition, ModelTurnCapability, ModelTurnRequest, ModelTurnResponse,
};
pub use provider::{
    AiProvider, ChatModelProtocol, ExecutionCapability, ProviderKind, RoutingPurpose,
};
