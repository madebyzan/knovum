//! Fast Guided Filter implementation for edge-preserving alpha matte refinement.
//!
//! Based on He, Sun & Tang (ECCV 2010 / TPAMI 2013) and Fast Guided Filter (He & Sun, 2015).
//! Computes edge-aligned alpha mattes by transferring structural edge gradients from a
//! full-resolution guidance image (e.g. grayscale luminance) onto a coarse segmentation mask.

/// Refines a coarse `[0, 1]` mask using a `[0, 1]` guidance image with O(N) separable box filtering.
pub fn fast_guided_filter(
    guidance: &[f32],
    mask: &[f32],
    width: usize,
    height: usize,
    radius: usize,
    eps: f32,
    subsample: usize,
) -> Vec<f32> {
    if width == 0 || height == 0 || guidance.len() != width * height || mask.len() != width * height
    {
        return mask.to_vec();
    }
    let s = subsample.max(1);
    let (sub_w, sub_h) = ((width / s).max(1), (height / s).max(1));
    let sub_r = (radius / s).max(1);

    let (i_sub, p_sub) = if s > 1 {
        (
            downsample(guidance, width, height, sub_w, sub_h),
            downsample(mask, width, height, sub_w, sub_h),
        )
    } else {
        (guidance.to_vec(), mask.to_vec())
    };

    let mean_i = box_filter_2d(&i_sub, sub_w, sub_h, sub_r);
    let mean_p = box_filter_2d(&p_sub, sub_w, sub_h, sub_r);

    let i_sq: Vec<f32> = i_sub.iter().map(|&v| v * v).collect();
    let i_p: Vec<f32> = i_sub.iter().zip(&p_sub).map(|(&i, &p)| i * p).collect();

    let mean_ii = box_filter_2d(&i_sq, sub_w, sub_h, sub_r);
    let mean_ip = box_filter_2d(&i_p, sub_w, sub_h, sub_r);

    let mut a = vec![0.0_f32; sub_w * sub_h];
    let mut b = vec![0.0_f32; sub_w * sub_h];
    for idx in 0..sub_w * sub_h {
        let var_i = mean_ii[idx] - mean_i[idx] * mean_i[idx];
        let cov_ip = mean_ip[idx] - mean_i[idx] * mean_p[idx];
        let a_val = cov_ip / (var_i + eps);
        a[idx] = a_val;
        b[idx] = mean_p[idx] - a_val * mean_i[idx];
    }

    let mean_a = box_filter_2d(&a, sub_w, sub_h, sub_r);
    let mean_b = box_filter_2d(&b, sub_w, sub_h, sub_r);

    let (a_full, b_full) = if s > 1 {
        (
            upsample_bilinear(&mean_a, sub_w, sub_h, width, height),
            upsample_bilinear(&mean_b, sub_w, sub_h, width, height),
        )
    } else {
        (mean_a, mean_b)
    };

    let mut output = vec![0.0_f32; width * height];
    for idx in 0..width * height {
        output[idx] = (a_full[idx] * guidance[idx] + b_full[idx]).clamp(0.0, 1.0);
    }
    output
}

fn box_filter_2d(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut temp = vec![0.0_f32; w * h];
    let mut dst = vec![0.0_f32; w * h];
    // Horizontal pass
    for y in 0..h {
        let row_offset = y * w;
        let mut acc = 0.0_f32;
        let right_bound = (w - 1).min(r);
        for x in 0..=right_bound {
            acc += src[row_offset + x];
        }
        for x in 0..w {
            let left = x.saturating_sub(r);
            let right = (x + r).min(w - 1);
            if x > 0 {
                if x + r < w {
                    acc += src[row_offset + x + r];
                }
                if x > r {
                    acc -= src[row_offset + x - r - 1];
                }
            }
            temp[row_offset + x] = acc / (right - left + 1) as f32;
        }
    }
    // Vertical pass
    for x in 0..w {
        let mut acc = 0.0_f32;
        let bot_bound = (h - 1).min(r);
        for y in 0..=bot_bound {
            acc += temp[y * w + x];
        }
        for y in 0..h {
            let top = y.saturating_sub(r);
            let bot = (y + r).min(h - 1);
            if y > 0 {
                if y + r < h {
                    acc += temp[(y + r) * w + x];
                }
                if y > r {
                    acc -= temp[(y - r - 1) * w + x];
                }
            }
            dst[y * w + x] = acc / (bot - top + 1) as f32;
        }
    }
    dst
}

fn downsample(src: &[f32], w: usize, h: usize, out_w: usize, out_h: usize) -> Vec<f32> {
    let mut dst = vec![0.0_f32; out_w * out_h];
    for y in 0..out_h {
        let src_y = ((y * h) / out_h).min(h - 1);
        for x in 0..out_w {
            let src_x = ((x * w) / out_w).min(w - 1);
            dst[y * out_w + x] = src[src_y * w + src_x];
        }
    }
    dst
}

fn upsample_bilinear(
    src: &[f32],
    in_w: usize,
    in_h: usize,
    out_w: usize,
    out_h: usize,
) -> Vec<f32> {
    let mut dst = vec![0.0_f32; out_w * out_h];
    let x_ratio = in_w as f32 / out_w as f32;
    let y_ratio = in_h as f32 / out_h as f32;
    for y in 0..out_h {
        let src_y = (y as f32 + 0.5) * y_ratio - 0.5;
        let y0 = (src_y.floor() as isize).clamp(0, (in_h - 1) as isize) as usize;
        let y1 = (y0 + 1).min(in_h - 1);
        let y_weight = (src_y - y0 as f32).clamp(0.0, 1.0);
        for x in 0..out_w {
            let src_x = (x as f32 + 0.5) * x_ratio - 0.5;
            let x0 = (src_x.floor() as isize).clamp(0, (in_w - 1) as isize) as usize;
            let x1 = (x0 + 1).min(in_w - 1);
            let x_weight = (src_x - x0 as f32).clamp(0.0, 1.0);

            let top = src[y0 * in_w + x0] * (1.0 - x_weight) + src[y0 * in_w + x1] * x_weight;
            let bot = src[y1 * in_w + x0] * (1.0 - x_weight) + src[y1 * in_w + x1] * x_weight;
            dst[y * out_w + x] = top * (1.0 - y_weight) + bot * y_weight;
        }
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guided_filter_preserves_bounds_and_smooths_edge() {
        let w = 8;
        let h = 8;
        // Step edge guidance
        let mut guidance = vec![0.0_f32; w * h];
        for x in 4..w {
            for y in 0..h {
                guidance[y * w + x] = 1.0;
            }
        }
        // Noisy coarse mask
        let mut mask = vec![0.1_f32; w * h];
        for x in 4..w {
            for y in 0..h {
                mask[y * w + x] = 0.9;
            }
        }
        let refined = fast_guided_filter(&guidance, &mask, w, h, 2, 1e-4, 1);
        assert_eq!(refined.len(), w * h);
        for &val in &refined {
            assert!((0.0..=1.0).contains(&val));
        }
        // Left side remains dark, right side remains light
        assert!(refined[0] < 0.25);
        assert!(refined[w - 1] > 0.75);
    }
}
