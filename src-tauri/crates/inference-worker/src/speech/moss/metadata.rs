use crate::artifact;
use serde::{de::DeserializeOwned, Deserialize};
use std::{io::Read, path::Path};

#[derive(Deserialize)]
pub(super) struct Manifest {
    pub(super) prompt_templates: Templates,
    pub(super) builtin_voices: Vec<Voice>,
}
#[derive(Deserialize)]
pub(super) struct Templates {
    pub(super) user_prompt_prefix_token_ids: Vec<i32>,
    pub(super) user_prompt_after_reference_token_ids: Vec<i32>,
    pub(super) assistant_prompt_prefix_token_ids: Vec<i32>,
}
#[derive(Deserialize)]
pub(super) struct Voice {
    pub(super) voice: String,
    pub(super) prompt_audio_codes: Vec<Vec<i32>>,
}
#[derive(Deserialize)]
pub(super) struct CodecMeta {
    pub(super) streaming_decode: Streaming,
}
#[derive(Deserialize)]
pub(super) struct Streaming {
    pub(super) transformer_offsets: Vec<Offset>,
    pub(super) attention_caches: Vec<Cache>,
}
#[derive(Deserialize)]
pub(super) struct Offset {
    pub(super) input_name: String,
    pub(super) output_name: String,
    pub(super) shape: Vec<i64>,
}
#[derive(Deserialize)]
pub(super) struct Cache {
    pub(super) offset_input_name: String,
    pub(super) offset_output_name: String,
    pub(super) cached_keys_input_name: String,
    pub(super) cached_keys_output_name: String,
    pub(super) cached_values_input_name: String,
    pub(super) cached_values_output_name: String,
    pub(super) cached_positions_input_name: String,
    pub(super) cached_positions_output_name: String,
    pub(super) offset_shape: Vec<i64>,
    pub(super) cache_shape: Vec<i64>,
    pub(super) positions_shape: Vec<i64>,
}

pub(super) fn read<T: DeserializeOwned>(directory: &Path, name: &str) -> Result<T, String> {
    let mut bytes = Vec::new();
    artifact::open_regular(&directory.join(name))?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 1_048_576 {
        return Err("model_corrupt: MOSS metadata exceeds its byte budget".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("model_corrupt: {error}"))
}

pub(super) fn count(shape: &[i64]) -> Result<usize, String> {
    if shape.is_empty() || shape.len() > 4 {
        return Err("model_corrupt: codec cache rank differs".into());
    }
    shape.iter().try_fold(1usize, |size, &dimension| {
        usize::try_from(dimension)
            .ok()
            .filter(|&dimension| dimension > 0)
            .and_then(|dimension| size.checked_mul(dimension))
            .filter(|&count| count <= 2_000_000)
            .ok_or("model_corrupt: codec cache exceeds its admitted shape".into())
    })
}
