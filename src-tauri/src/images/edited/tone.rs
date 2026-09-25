use image::Rgba;

use super::contract::Adjustments;

pub(super) struct ToneAdjuster {
    active: bool,
    exposure: f64,
    contrast: f64,
    shadows: f64,
    highlights: f64,
    blacks: f64,
    whites: f64,
    saturation: f64,
    vibrance: f64,
    warmth: f64,
    tint: f64,
    vignette: f64,
    soft_light: f64,
}

impl ToneAdjuster {
    pub(super) fn new(value: &Adjustments) -> Self {
        Self {
            active: [
                value.exposure,
                value.contrast,
                value.shadows,
                value.highlights,
                value.blacks,
                value.whites,
                value.saturation,
                value.vibrance,
                value.warmth,
                value.tint,
                value.vignette,
                value.soft_light,
            ]
            .iter()
            .any(|value| *value != 0.0),
            exposure: 2.0_f64.powf((value.exposure / 100.0) * 2.0),
            contrast: 2.0_f64.powf((value.contrast / 100.0) * 1.5),
            shadows: value.shadows / 100.0 * 0.55,
            highlights: value.highlights / 100.0 * 0.55,
            blacks: value.blacks / 100.0 * 0.35,
            whites: value.whites / 100.0 * 0.35,
            saturation: 1.0 + value.saturation / 100.0,
            vibrance: value.vibrance / 100.0 * 0.8,
            warmth: value.warmth / 100.0 * 0.12,
            tint: value.tint / 100.0 * 0.08,
            vignette: value.vignette / 100.0,
            soft_light: value.soft_light / 100.0 * 0.6,
        }
    }

    pub(super) fn apply(&self, pixel: &mut Rgba<u8>, x: u32, y: u32, width: u32, height: u32) {
        if !self.active || pixel[3] == 0 {
            return;
        }
        let mut channels = [0.0; 3];
        for index in 0..3 {
            channels[index] = contrast_channel(
                f64::from(pixel[index]) / 255.0 * self.exposure,
                self.contrast,
            );
        }
        let luminance = 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
        let weights = [
            1.0 - smoothstep(0.08, 0.68, luminance),
            smoothstep(0.32, 0.92, luminance),
            1.0 - smoothstep(0.0, 0.3, luminance),
            smoothstep(0.7, 1.0, luminance),
        ];
        for channel in &mut channels {
            *channel = adjust_range(*channel, self.shadows, weights[0]);
            *channel = adjust_range(*channel, self.highlights, weights[1]);
            *channel = adjust_range(*channel, self.blacks, weights[2]);
            *channel = adjust_range(*channel, self.whites, weights[3]);
        }
        let luminance = 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
        let chroma = channels.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - channels.iter().copied().fold(f64::INFINITY, f64::min);
        let color_strength = self.saturation * (1.0 + self.vibrance * (1.0 - chroma));
        for (index, channel) in channels.iter_mut().enumerate() {
            let warmth = match index {
                0 => self.warmth,
                2 => -self.warmth,
                _ => 0.0,
            };
            let tint = if index == 1 { -self.tint } else { self.tint };
            *channel = (luminance + (*channel - luminance) * color_strength + warmth + tint)
                .clamp(0.0, 1.0);
            *channel += self.soft_light * *channel * (1.0 - *channel);
        }
        if self.vignette != 0.0 {
            let dx = (f64::from(x) + 0.5 - f64::from(width) / 2.0) / (f64::from(width) / 2.0);
            let dy = (f64::from(y) + 0.5 - f64::from(height) / 2.0) / (f64::from(height) / 2.0);
            let edge = smoothstep(0.2, 1.1, dx.hypot(dy)) * self.vignette.abs() * 0.75;
            for channel in &mut channels {
                *channel = if self.vignette > 0.0 {
                    *channel * (1.0 - edge)
                } else {
                    *channel + (1.0 - *channel) * edge
                };
            }
        }
        for index in 0..3 {
            pixel[index] = to_byte(channels[index].clamp(0.0, 1.0) * 255.0);
        }
    }
}

fn contrast_channel(value: f64, contrast: f64) -> f64 {
    ((value - 0.5) * contrast + 0.5).clamp(0.0, 1.0)
}

fn adjust_range(value: f64, amount: f64, weight: f64) -> f64 {
    let room = if amount >= 0.0 { 1.0 - value } else { value };
    (value + room * amount * weight).clamp(0.0, 1.0)
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let position = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    position * position * (3.0 - 2.0 * position)
}

pub(super) fn to_byte(value: f64) -> u8 {
    value.clamp(0.0, 255.0).round_ties_even() as u8
}
