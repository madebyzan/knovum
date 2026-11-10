//! Edge color decontamination (defringing) for alpha-composited boundaries.
//!
//! Semi-transparent boundary pixels ($0 < \alpha < 0.95$) typically suffer from background
//! color bleed (halos/fringing) from the original backdrop. This module recovers genuine
//! foreground colors along soft edges by propagating nearby interior foreground RGB.

use image::RgbaImage;

const OPAQUE_THRESHOLD: u8 = 242;
const SEARCH_RADIUS: i32 = 3;

/// Suppresses background color bleed on semi-transparent edge pixels.
pub fn decontaminate_edges(source: &RgbaImage, alpha_matte: &[u8]) -> RgbaImage {
    let (width, height) = source.dimensions();
    let (w, h) = (width as usize, height as usize);
    if alpha_matte.len() != w * h {
        return source.clone();
    }

    let mut result = source.clone();
    let src_pixels = source.as_raw();
    let dst_pixels: &mut [u8] = &mut result;

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let idx = (y as usize * w + x as usize) * 4;
            let alpha = alpha_matte[y as usize * w + x as usize];

            // Only process transition edge pixels with partial transparency
            if alpha > 0 && alpha < OPAQUE_THRESHOLD {
                if let Some([fg_r, fg_g, fg_b]) =
                    find_nearest_opaque(src_pixels, alpha_matte, w, h, x, y)
                {
                    let weight = f32::from(OPAQUE_THRESHOLD - alpha) / f32::from(OPAQUE_THRESHOLD);
                    for c in 0..3 {
                        let orig = f32::from(src_pixels[idx + c]);
                        let fg = f32::from(match c {
                            0 => fg_r,
                            1 => fg_g,
                            _ => fg_b,
                        });
                        dst_pixels[idx + c] = ((orig * (1.0 - weight) + fg * weight) + 0.5) as u8;
                    }
                }
            }
        }
    }
    result
}

fn find_nearest_opaque(
    src: &[u8],
    alpha: &[u8],
    w: usize,
    h: usize,
    cx: i32,
    cy: i32,
) -> Option<[u8; 3]> {
    let mut best_dist_sq = i32::MAX;
    let mut best_rgb = None;

    for dy in -SEARCH_RADIUS..=SEARCH_RADIUS {
        let ny = cy + dy;
        if ny < 0 || ny >= h as i32 {
            continue;
        }
        for dx in -SEARCH_RADIUS..=SEARCH_RADIUS {
            let nx = cx + dx;
            if nx < 0 || nx >= w as i32 {
                continue;
            }
            let n_idx = ny as usize * w + nx as usize;
            if alpha[n_idx] >= OPAQUE_THRESHOLD {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    let p_idx = n_idx * 4;
                    best_rgb = Some([src[p_idx], src[p_idx + 1], src[p_idx + 2]]);
                    if dist_sq <= 1 {
                        return best_rgb;
                    }
                }
            }
        }
    }
    best_rgb
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_decontaminate_edges_pulls_color_from_opaque_neighbor() {
        let mut img = RgbaImage::new(4, 1);
        // Pixel 0: Opaque Red
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        // Pixel 1: Half-transparent contaminated with white background
        img.put_pixel(1, 0, Rgba([255, 200, 200, 128]));
        // Pixel 2 & 3: Transparent
        img.put_pixel(2, 0, Rgba([255, 255, 255, 0]));
        img.put_pixel(3, 0, Rgba([255, 255, 255, 0]));

        let alpha = vec![255, 128, 0, 0];
        let cleaned = decontaminate_edges(&img, &alpha);

        let p1 = cleaned.get_pixel(1, 0);
        // Green and Blue channels should have been pulled closer to 0 (the red foreground)
        assert!(p1[1] < 200);
        assert!(p1[2] < 200);
    }
}
