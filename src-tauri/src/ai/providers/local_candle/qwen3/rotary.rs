use candle_core::{DType, Device, Result, Tensor, D};

use super::config::Config;

pub struct RotaryEmbedding {
    inv_freq: Tensor,
    attention_factor: f32,
}

impl RotaryEmbedding {
    pub fn new(cfg: &Config, device: &Device) -> Result<Self> {
        let base = cfg.rope_theta;
        let dim = cfg.head_dim();
        assert!(dim.is_multiple_of(2), "head_dim must be even, got {dim}");

        let t = Tensor::arange_step(0u32, dim as u32, 2u32, device)?.to_dtype(DType::F32)?;
        let dim_t = Tensor::new(dim as f32, device)?;
        let exponent = t.broadcast_div(&dim_t)?;

        let ln_base = (base as f32).ln();
        let ln_base_t = Tensor::new(ln_base, device)?;
        let inv_freq = exponent.broadcast_mul(&ln_base_t.neg()?)?.exp()?;

        Ok(Self {
            inv_freq,
            attention_factor: 1.0,
        })
    }

    pub fn forward(&self, xs: &Tensor, position_ids: &Tensor) -> Result<(Tensor, Tensor)> {
        let dev = xs.device();
        let inv_freq = self.inv_freq.to_device(dev)?.to_dtype(DType::F32)?;
        let d2 = inv_freq.dims1()?;

        let (b, t) = position_ids.dims2()?;
        let positions = position_ids
            .to_device(dev)?
            .to_dtype(DType::F32)?
            .contiguous()?;

        let inv_freq_expanded = inv_freq
            .reshape((1, d2, 1))?
            .expand((b, d2, 1))?
            .contiguous()?;
        let positions_expanded = positions.reshape((b, 1, t))?.contiguous()?;

        let freqs = inv_freq_expanded
            .matmul(&positions_expanded)?
            .transpose(1, 2)?
            .contiguous()?;

        let emb = Tensor::cat(&[&freqs, &freqs], 2)?;
        let scale = Tensor::new(self.attention_factor, dev)?;
        let cos = emb.cos()?.broadcast_mul(&scale)?;
        let sin = emb.sin()?.broadcast_mul(&scale)?;

        let out_dtype = xs.dtype();
        Ok((cos.to_dtype(out_dtype)?, sin.to_dtype(out_dtype)?))
    }
}

pub fn apply_rotary_pos_emb(
    q: &Tensor,
    k: &Tensor,
    cos: &Tensor,
    sin: &Tensor,
) -> Result<(Tensor, Tensor)> {
    let cos_u = cos.unsqueeze(1)?;
    let sin_u = sin.unsqueeze(1)?;

    let q_embed = (q.broadcast_mul(&cos_u)? + rotate_half(q)?.broadcast_mul(&sin_u)?)?;
    let k_embed = (k.broadcast_mul(&cos_u)? + rotate_half(k)?.broadcast_mul(&sin_u)?)?;
    Ok((q_embed, k_embed))
}

fn rotate_half(x: &Tensor) -> Result<Tensor> {
    let d = x
        .dims()
        .last()
        .copied()
        .ok_or_else(|| candle_core::Error::Msg("empty dims".into()))?;
    assert!(d % 2 == 0, "rotate_half requires even last dim, got {d}");
    let half = d / 2;

    let x1 = x.narrow(D::Minus1, 0, half)?;
    let x2 = x.narrow(D::Minus1, half, half)?;
    let nx2 = x2.neg()?;
    Tensor::cat(&[&nx2, &x1], x.rank() - 1)
}
