use crate::{artifact, cancellation::Cancellation};
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen3::{Config, Model};
use knovum_inference_contracts::{
    embedding_wire::{EmbeddingEncodeParams, EmbeddingEncoding, EmbeddingRole, EMBEDDING_SPEC_ID},
    execution_profile,
    runtime::RuntimeInitializeParams,
};
use std::path::Path;
use tokenizers::Tokenizer;

const QUERY_PROMPT: &str = "Instruct: Given a knowledge-space search query, retrieve relevant passages from the user's documents, notes, media transcripts, code, and web captures.\nQuery:";

pub(crate) struct Embedding {
    base: Model,
    tokenizer: Tokenizer,
    device: Device,
}

/// Admits the frozen embedding identity and execution profile and returns the snapshot it names.
///
/// This is a pure function of the initialization message, so an obsolete identity or an
/// unadmitted thread count is rejected before any weight is read.
fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    let device = params.device_profile.device.as_str();
    let threads = params.device_profile.cpu_threads;
    if params.capability_spec_id != EMBEDDING_SPEC_ID
        || !matches!(device, "cpu" | "mps")
        || device == "mps" && !cfg!(all(target_os = "macos", target_arch = "aarch64"))
        || !(1..=8).contains(&threads)
        || params.device_profile.execution_profile_id
            != execution_profile::embedding(device, threads)
    {
        return Err("runtime_mismatch: invalid embedding initialization".into());
    }
    params
        .model_paths
        .embedding
        .as_deref()
        .ok_or_else(|| "model_missing: embedding path is missing".into())
}

impl Embedding {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        let directory = Path::new(admit(params)?);
        let device = match params.device_profile.device.as_str() {
            "cpu" => Device::Cpu,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            "mps" => Device::new_metal(0).map_err(model_error)?,
            _ => return Err("unsupported_device: embedding device is unavailable".into()),
        };
        let config_path = directory.join("config.json");
        let config: Config =
            serde_json::from_reader(artifact::open_regular(&config_path)?).map_err(model_error)?;
        if config.hidden_size != 1024
            || config.num_hidden_layers != 28
            || config.vocab_size != 151669
        {
            return Err("model_corrupt: unexpected Qwen3 embedding configuration".into());
        }
        let path = directory.join("model.safetensors");
        artifact::open_regular(&path)?;
        // The host verified the immutable snapshot and retains its resource lease through child exit.
        let builder = unsafe { VarBuilder::from_mmaped_safetensors(&[path], DType::F32, &device) }
            .map_err(model_error)?;
        let builder = builder.rename_f(|name| {
            name.strip_prefix("model.")
                .expect("Qwen3 model queries use its model prefix")
                .to_owned()
        });
        let base = Model::new(&config, builder).map_err(model_error)?;
        let path = directory.join("tokenizer.json");
        artifact::open_regular(&path)?;
        let mut tokenizer = Tokenizer::from_file(path).map_err(model_error)?;
        tokenizer.with_padding(None);
        tokenizer.with_truncation(None).map_err(model_error)?;
        Ok(Self {
            base,
            tokenizer,
            device,
        })
    }

    pub(crate) fn encode(
        &self,
        params: EmbeddingEncodeParams,
        cancel: &Cancellation,
    ) -> Result<EmbeddingEncoding, String> {
        cancel.check()?;
        if params.texts.is_empty()
            || params.texts.len() > 32
            || params.embedding_spec_id != params.model_spec_id
            || params.model_spec_id != EMBEDDING_SPEC_ID
        {
            return Err("invalid_input: invalid embedding batch or identity".into());
        }
        let mut tokens = Vec::with_capacity(params.texts.len());
        let mut input_tokens = 0u32;
        for text in &params.texts {
            cancel.check()?;
            let encoded = self
                .tokenizer
                .encode(prepare(text, params.role), true)
                .map_err(|error| format!("invalid_input: {error}"))?;
            let ids = encoded.get_ids().to_vec();
            if ids.is_empty() || ids.len() > 2048 {
                return Err("invalid_input: embedding input must contain 1 to 2048 tokens".into());
            }
            input_tokens += ids.len() as u32;
            tokens.push(ids);
        }
        let mut vectors = Vec::with_capacity(tokens.len());
        for ids in tokens {
            cancel.check()?;
            // Each text starts with empty KV state. Model cloning shares immutable tensors only.
            // Processing one unpadded sequence makes last-token pooling independent of batch padding.
            let mut model = self.base.clone();
            let input = Tensor::new(ids.as_slice(), &self.device)
                .and_then(|input| input.unsqueeze(0))
                .map_err(model_error)?;
            let output = model.forward(&input, 0).map_err(model_error)?;
            cancel.check()?;
            let mut vector = output
                .i((0, ids.len() - 1, ..))
                .and_then(|last| last.to_vec1::<f32>())
                .map_err(model_error)?;
            normalize(&mut vector)?;
            vectors.push(vector);
        }
        Ok(EmbeddingEncoding {
            request_id: params.request_id,
            attempt_epoch: params.attempt_epoch,
            embedding_spec_id: params.embedding_spec_id,
            vectors,
            dimensions: 1024,
            input_tokens,
        })
    }
}

fn prepare(text: &str, role: EmbeddingRole) -> String {
    match role {
        EmbeddingRole::Query => format!("{QUERY_PROMPT}{text}"),
        EmbeddingRole::Document => text.into(),
    }
}
fn normalize(vector: &mut [f32]) -> Result<(), String> {
    if vector.len() != 1024 || vector.iter().any(|value| !value.is_finite()) {
        return Err("invalid_output: embedding dimensions or values differ".into());
    }
    let norm = vector
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= f64::EPSILON {
        return Err("invalid_output: embedding norm is invalid".into());
    }
    for value in vector {
        *value = (f64::from(*value) / norm) as f32;
    }
    Ok(())
}
fn model_error(error: impl std::fmt::Display) -> String {
    format!("model_corrupt: {error}")
}

#[cfg(test)]
mod tests;
