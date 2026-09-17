use candle_core::{Result, Tensor};
use candle_nn::VarBuilder;

use super::{attention::Attention, config::Config, mlp::Mlp, norm::RmsNorm};

pub struct DecoderLayer {
    self_attn: Attention,
    mlp: Mlp,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
}

impl DecoderLayer {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            self_attn: Attention::new(cfg, vb.pp("self_attn"))?,
            mlp: Mlp::new(cfg, vb.pp("mlp"))?,
            input_layernorm: RmsNorm::new(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("input_layernorm"),
            )?,
            post_attention_layernorm: RmsNorm::new(
                cfg.hidden_size,
                cfg.rms_norm_eps,
                vb.pp("post_attention_layernorm"),
            )?,
        })
    }

    pub fn forward(
        &self,
        hidden_states: &Tensor,
        attention_mask: Option<&Tensor>,
        position_embeddings: (&Tensor, &Tensor),
    ) -> Result<Tensor> {
        let residual = hidden_states.clone();
        let hs = hidden_states.apply(&self.input_layernorm)?;
        let hs = self
            .self_attn
            .forward(&hs, position_embeddings, attention_mask)?;
        let hs = (residual + hs)?;

        let residual = hs.clone();
        let hs2 = hs.apply(&self.post_attention_layernorm)?;
        let hs2 = hs2.apply(&self.mlp)?;
        residual + hs2
    }
}
