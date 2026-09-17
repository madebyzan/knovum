use candle_core::{Result, Tensor, D};
use candle_nn::{linear, linear_no_bias, Linear, VarBuilder};

use super::{config::Config, norm::RmsNorm, rotary::apply_rotary_pos_emb};

pub struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    q_norm: RmsNorm,
    k_norm: RmsNorm,
    num_heads: usize,
    num_kv_heads: usize,
    num_kv_groups: usize,
    head_dim: usize,
    scaling: f32,
}

impl Attention {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let head_dim = cfg.head_dim();
        let num_heads = cfg.num_attention_heads;
        let num_kv_heads = cfg.num_key_value_heads;
        let num_kv_groups = cfg.num_kv_groups();
        assert!(
            num_heads.is_multiple_of(num_kv_heads),
            "num_heads must be multiple of num_kv_heads"
        );

        let q_out = num_heads * head_dim;
        let kv_out = num_kv_heads * head_dim;

        let q_proj = if cfg.attention_bias {
            linear(cfg.hidden_size, q_out, vb.pp("q_proj"))?
        } else {
            linear_no_bias(cfg.hidden_size, q_out, vb.pp("q_proj"))?
        };
        let k_proj = if cfg.attention_bias {
            linear(cfg.hidden_size, kv_out, vb.pp("k_proj"))?
        } else {
            linear_no_bias(cfg.hidden_size, kv_out, vb.pp("k_proj"))?
        };
        let v_proj = if cfg.attention_bias {
            linear(cfg.hidden_size, kv_out, vb.pp("v_proj"))?
        } else {
            linear_no_bias(cfg.hidden_size, kv_out, vb.pp("v_proj"))?
        };
        let o_proj = if cfg.attention_bias {
            linear(q_out, cfg.hidden_size, vb.pp("o_proj"))?
        } else {
            linear_no_bias(q_out, cfg.hidden_size, vb.pp("o_proj"))?
        };

        let q_norm = RmsNorm::new(head_dim, cfg.rms_norm_eps, vb.pp("q_norm"))?;
        let k_norm = RmsNorm::new(head_dim, cfg.rms_norm_eps, vb.pp("k_norm"))?;

        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            q_norm,
            k_norm,
            num_heads,
            num_kv_heads,
            num_kv_groups,
            head_dim,
            scaling: (head_dim as f32).powf(-0.5),
        })
    }

    pub fn forward(
        &self,
        hidden_states: &Tensor,
        position_embeddings: (&Tensor, &Tensor),
        attention_mask: Option<&Tensor>,
    ) -> Result<Tensor> {
        let (b, t, _h) = hidden_states.dims3()?;
        let d = self.head_dim;

        let q = hidden_states
            .apply(&self.q_proj)?
            .reshape((b, t, self.num_heads, d))?;
        let q = q.apply(&self.q_norm)?.transpose(1, 2)?;

        let k = hidden_states
            .apply(&self.k_proj)?
            .reshape((b, t, self.num_kv_heads, d))?;
        let k = k.apply(&self.k_norm)?.transpose(1, 2)?;

        let v = hidden_states
            .apply(&self.v_proj)?
            .reshape((b, t, self.num_kv_heads, d))?
            .transpose(1, 2)?;

        let (cos, sin) = position_embeddings;
        let (q, k) = apply_rotary_pos_emb(&q, &k, cos, sin)?;

        let k = repeat_kv(&k, self.num_kv_groups)?;
        let v = repeat_kv(&v, self.num_kv_groups)?;

        let kt = k.transpose(2, 3)?;
        let mut attn = q.matmul(&kt)?;

        let scale = Tensor::new(self.scaling, attn.device())?.to_dtype(attn.dtype())?;
        attn = attn.broadcast_mul(&scale)?;

        if let Some(mask) = attention_mask {
            attn = attn.broadcast_add(&mask.to_dtype(attn.dtype())?)?;
        }

        let attn = candle_nn::ops::softmax(&attn, D::Minus1)?;
        let out = attn.matmul(&v)?;
        let out = out.transpose(1, 2)?.reshape((b, t, self.num_heads * d))?;
        out.apply(&self.o_proj)
    }
}

fn repeat_kv(x: &Tensor, n_rep: usize) -> Result<Tensor> {
    if n_rep == 1 {
        return Ok(x.clone());
    }
    let (b, n_kv, t, d) = x.dims4()?;
    let x = x.unsqueeze(2)?;
    let x = x.broadcast_as((b, n_kv, n_rep, t, d))?;
    x.reshape((b, n_kv * n_rep, t, d))
}
