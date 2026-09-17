use candle_core::{DType, Module, Result, Tensor, D};
use candle_nn::VarBuilder;

pub struct RmsNorm {
    weight: Tensor,
    eps: f64,
}

impl RmsNorm {
    pub fn new(dim: usize, eps: f64, vb: VarBuilder) -> Result<Self> {
        let weight = vb.get((dim,), "weight")?;
        Ok(Self { weight, eps })
    }
}

impl Module for RmsNorm {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let in_dtype = xs.dtype();
        let xs_f = xs.to_dtype(DType::F32)?;
        let var = xs_f.powf(2.0)?.mean_keepdim(D::Minus1)?;
        let eps_t = Tensor::new(self.eps as f32, xs.device())?;
        let var_eps = var.broadcast_add(&eps_t)?;
        let inv_rms = var_eps.sqrt()?.recip()?;
        let normed = xs_f.broadcast_mul(&inv_rms)?;
        let normed = normed.to_dtype(in_dtype)?;
        let w = self.weight.to_dtype(in_dtype)?;
        normed.broadcast_mul(&w)
    }
}
