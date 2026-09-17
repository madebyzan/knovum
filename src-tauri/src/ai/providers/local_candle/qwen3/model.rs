use candle_core::{Device, Module, Result, Tensor};
use candle_nn::{embedding, Embedding, VarBuilder};

use super::{config::Config, decoder::DecoderLayer, norm::RmsNorm, rotary::RotaryEmbedding};

pub struct Model {
    embed_tokens: Embedding,
    layers: Vec<DecoderLayer>,
    norm: RmsNorm,
    rotary_emb: RotaryEmbedding,
    device: Device,
}

impl Model {
    pub fn new(cfg: Config, vb: VarBuilder) -> Result<Self> {
        let device = vb.device().clone();
        let embed_tokens = embedding(cfg.vocab_size, cfg.hidden_size, vb.pp("embed_tokens"))?;

        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for i in 0..cfg.num_hidden_layers {
            layers.push(DecoderLayer::new(&cfg, vb.pp(format!("layers.{i}")))?);
        }

        let norm = RmsNorm::new(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("norm"))?;
        let rotary_emb = RotaryEmbedding::new(&cfg, vb.device())?;

        Ok(Self {
            embed_tokens,
            layers,
            norm,
            rotary_emb,
            device,
        })
    }

    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask_4d: Option<&Tensor>,
    ) -> Result<Tensor> {
        let (b, t) = input_ids.dims2()?;
        let mut hs = self.embed_tokens.forward(input_ids)?;

        let pos_1d = Tensor::arange(0u32, t as u32, hs.device())?;
        let position_ids = pos_1d.unsqueeze(0)?.expand((b, t))?.contiguous()?;

        let (cos, sin) = self.rotary_emb.forward(&hs, &position_ids)?;

        for layer in self.layers.iter() {
            hs = layer.forward(&hs, attention_mask_4d, (&cos, &sin))?;
        }

        hs.apply(&self.norm)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
}
