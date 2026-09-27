use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use candle_core::{DType, Device};
use candle_nn::VarBuilder;

use super::qwen3::{Config, EmbeddingOutput, Qwen3Model, Qwen3TextEmbedding};
use crate::local_resources::{models::catalog::ModelKind, preparation::PreparedModels};

static ENGINE: OnceLock<Mutex<Option<Arc<Qwen3TextEmbedding>>>> = OnceLock::new();
static INFERENCE: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();

fn engine_lock() -> &'static Mutex<Option<Arc<Qwen3TextEmbedding>>> {
    ENGINE.get_or_init(|| Mutex::new(None))
}

pub fn unload() -> Result<(), String> {
    let mut guard = engine_lock()
        .lock()
        .map_err(|_| "Embedding engine lock was poisoned".to_string())?;
    *guard = None;
    Ok(())
}

fn get_or_load(resources: &PreparedModels) -> Result<Arc<Qwen3TextEmbedding>, String> {
    let mut guard = engine_lock()
        .lock()
        .map_err(|_| "Embedding engine lock was poisoned".to_string())?;
    if let Some(existing) = guard.as_ref() {
        return Ok(Arc::clone(existing));
    }

    let config_path = resources.path(ModelKind::TextEmbeddingConfig)?;
    let tokenizer_path = resources.path(ModelKind::TextEmbeddingTokenizer)?;
    let weights_path = resources.path(ModelKind::TextEmbeddingWeights)?;

    let model = Arc::new(load_model(config_path, tokenizer_path, weights_path)?);
    *guard = Some(Arc::clone(&model));
    Ok(model)
}

fn load_model(
    config_path: &Path,
    tokenizer_path: &Path,
    weights_path: &Path,
) -> Result<Qwen3TextEmbedding, String> {
    let config_bytes = fs::read(config_path)
        .map_err(|error| format!("Failed to read Qwen3 embedding config: {error}"))?;
    let config: Config = serde_json::from_slice(&config_bytes)
        .map_err(|error| format!("Failed to parse Qwen3 embedding config: {error}"))?;

    let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
        .map_err(|error| format!("Failed to load Qwen3 embedding tokenizer: {error}"))?;

    let device = Device::Cpu;
    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)
            .map_err(|error| format!("Failed to mmap Qwen3 weights: {error}"))?
    };

    let model = Qwen3Model::new(config, vb)
        .map_err(|error| format!("Failed to initialize Qwen3 model: {error}"))?;

    Ok(Qwen3TextEmbedding::new(model, tokenizer))
}

pub async fn embed(
    texts: Vec<String>,
    resources: PreparedModels,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<EmbeddingOutput, String> {
    let queue = INFERENCE
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(1)))
        .clone();
    let permit = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err("knowledge-indexing-cancelled".into()),
        permit = queue.acquire_owned() => permit.map_err(|error| format!("Embedding inference queue closed: {error}"))?,
    };
    tokio::task::spawn_blocking(move || {
        // Keep ownership inside the blocking task: cancelling its caller cannot allow
        // another inference to overlap a computation that is still running.
        let _permit = permit;
        let embedder = get_or_load(&resources)?;
        embedder
            .embed(&texts)
            .map_err(|error| format!("Qwen3 local embedding inference failed: {error}"))
    })
    .await
    .map_err(|error| format!("Embedding task join failed: {error}"))?
}
