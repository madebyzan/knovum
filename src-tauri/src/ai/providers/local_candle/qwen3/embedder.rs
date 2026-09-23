use candle_core::{DType, IndexOp, Result, Tensor};
use tokenizers::{Encoding, PaddingDirection, Tokenizer, TruncationParams};

use super::model::Model;

pub struct TextEmbedding {
    model: Model,
    tokenizer: Tokenizer,
}

pub struct EmbeddingOutput {
    pub vectors: Vec<Vec<f32>>,
    pub input_tokens: usize,
}

const MAX_INFERENCE_ROWS: usize = 8;
const MAX_PADDED_TOKENS: usize = 2048;
const MAX_ATTENTION_CELLS: usize = 2048 * 2048;

impl TextEmbedding {
    pub fn new(model: Model, mut tokenizer: Tokenizer) -> Self {
        tokenizer.with_padding(None);
        let _ = tokenizer.with_truncation(Some(TruncationParams {
            max_length: 2048,
            ..Default::default()
        }));
        Self { model, tokenizer }
    }

    pub fn embed(&self, texts: &[String]) -> Result<EmbeddingOutput> {
        if texts.is_empty() {
            return Ok(EmbeddingOutput {
                vectors: vec![],
                input_tokens: 0,
            });
        }

        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;

        let input_tokens = encodings.iter().map(Encoding::len).sum();
        let mut ordered: Vec<_> = encodings.into_iter().enumerate().collect();
        ordered.sort_by_key(|(_, encoding)| encoding.len());
        let mut output = vec![None; texts.len()];
        let mut start = 0;
        while start < ordered.len() {
            let mut end = start + 1;
            while end < ordered.len() {
                let length = ordered[end].1.len();
                let count = end - start + 1;
                if count > MAX_INFERENCE_ROWS
                    || count * length > MAX_PADDED_TOKENS
                    || count * length * length > MAX_ATTENTION_CELLS
                {
                    break;
                }
                end += 1;
            }
            let batch = &ordered[start..end];
            let length = batch.last().unwrap().1.len();
            let mut encodings: Vec<_> =
                batch.iter().map(|(_, encoding)| encoding.clone()).collect();
            for encoding in &mut encodings {
                encoding.pad(length, 0, 0, "[PAD]", PaddingDirection::Left);
            }
            let started = std::time::Instant::now();
            let vectors = self.embed_batch(&encodings)?;
            log::info!(
                "embedding_batch rows={} tokens={} elapsed_ms={}",
                batch.len(),
                length,
                started.elapsed().as_millis()
            );
            for ((index, _), vector) in batch.iter().zip(vectors) {
                output[*index] = Some(vector);
            }
            start = end;
        }
        let vectors = output
            .into_iter()
            .map(|vector| {
                vector.ok_or_else(|| {
                    candle_core::Error::Msg("Embedding batch omitted an input".into())
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(EmbeddingOutput {
            vectors,
            input_tokens,
        })
    }

    fn embed_batch(&self, encodings: &[Encoding]) -> Result<Vec<Vec<f32>>> {
        let batch_size = encodings.len();
        let seq_len = encodings[0].len();

        let mut input_ids_vec: Vec<u32> = Vec::with_capacity(batch_size * seq_len);
        let mut attention_mask_vec: Vec<f32> = Vec::with_capacity(batch_size * seq_len);

        for enc in encodings {
            input_ids_vec.extend(enc.get_ids().iter().copied());
            attention_mask_vec.extend(enc.get_attention_mask().iter().map(|&m| m as f32));
        }

        let device = self.model.device();
        let input_ids = Tensor::from_vec(input_ids_vec, (batch_size, seq_len), device)?;
        let attention_mask_2d =
            Tensor::from_vec(attention_mask_vec, (batch_size, seq_len), device)?;

        let attention_mask_4d = build_attention_mask_4d(&attention_mask_2d)?;
        let hidden = self.model.forward(&input_ids, Some(&attention_mask_4d))?;

        let pooled = hidden.i((.., seq_len - 1))?;
        let normalized = l2_normalize(&pooled)?;

        let normalized = normalized.to_dtype(DType::F32)?;
        normalized.to_vec2::<f32>()
    }
}

fn build_attention_mask_4d(attention_mask_2d: &Tensor) -> Result<Tensor> {
    let (batch_size, seq_len) = attention_mask_2d.dims2()?;
    let device = attention_mask_2d.device();
    let mask_value = -1e4f32;

    let mut causal_values = vec![0.0f32; seq_len * seq_len];
    for i in 0..seq_len {
        for j in (i + 1)..seq_len {
            causal_values[i * seq_len + j] = mask_value;
        }
    }
    let causal_mask = Tensor::from_vec(causal_values, (1, 1, seq_len, seq_len), device)?;

    let pad_mask_expanded = attention_mask_2d.unsqueeze(1)?.unsqueeze(2)?;
    let pad_mask_expanded = pad_mask_expanded.expand((batch_size, 1, seq_len, seq_len))?;
    let pad_mask_f32 = pad_mask_expanded.to_dtype(DType::F32)?;
    let ones = Tensor::ones_like(&pad_mask_f32)?;
    let inverted_mask = ones.sub(&pad_mask_f32)?;
    let mask_val_t = Tensor::new(&[mask_value], device)?;
    let pad_additive = inverted_mask.broadcast_mul(&mask_val_t)?;

    let causal_broadcast = causal_mask.broadcast_as((batch_size, 1, seq_len, seq_len))?;
    causal_broadcast.add(&pad_additive)
}

fn l2_normalize(xs: &Tensor) -> Result<Tensor> {
    let sum_sq = xs.sqr()?.sum_keepdim(1)?;
    let eps_tensor = Tensor::new(&[1e-12f32], xs.device())?
        .to_dtype(sum_sq.dtype())?
        .broadcast_as(sum_sq.shape())?;
    let norm = sum_sq.add(&eps_tensor)?.sqrt()?;
    xs.broadcast_div(&norm)
}
