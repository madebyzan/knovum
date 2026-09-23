import type { ImageAdjustments } from "./image-edit";
import { applyImageDetail } from "./image-detail";
import { applyImageGlow } from "./image-glow";

const BYTE_MAX = 255;

export function applyImageTone(
  source: Uint8ClampedArray,
  adjustments: ImageAdjustments,
  width: number,
  height: number,
  imageRect: { left: number; top: number; width: number; height: number },
): Uint8ClampedArray {
  const output = new Uint8ClampedArray(source);
  if (Object.entries(adjustments).every(([key, value]) => ["sharpness", "clarity", "grain", "glow"].includes(key) || value === 0)) {
    return finishSpatialAdjustments(output, width, height, adjustments, imageRect);
  }
  const exposure = 2 ** ((adjustments.exposure / 100) * 2);
  const contrast = 2 ** ((adjustments.contrast / 100) * 1.5);
  const shadows = (adjustments.shadows / 100) * 0.55;
  const highlights = (adjustments.highlights / 100) * 0.55;
  const whites = (adjustments.whites / 100) * 0.35;
  const blacks = (adjustments.blacks / 100) * 0.35;
  const saturation = 1 + adjustments.saturation / 100;
  const vibrance = (adjustments.vibrance / 100) * 0.8;
  const warmth = (adjustments.warmth / 100) * 0.12;
  const tint = (adjustments.tint / 100) * 0.08;
  const softLight = (adjustments.softLight / 100) * 0.6;

  for (let offset = 0; offset < output.length; offset += 4) {
    if (output[offset + 3] === 0) continue;
    let red = contrastChannel((output[offset]! / BYTE_MAX) * exposure, contrast);
    let green = contrastChannel((output[offset + 1]! / BYTE_MAX) * exposure, contrast);
    let blue = contrastChannel((output[offset + 2]! / BYTE_MAX) * exposure, contrast);
    const luminance = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    const shadowWeight = 1 - smoothstep(0.08, 0.68, luminance);
    const highlightWeight = smoothstep(0.32, 0.92, luminance);
    const blackWeight = 1 - smoothstep(0, 0.3, luminance);
    const whiteWeight = smoothstep(0.7, 1, luminance);

    red = toneChannel(red, shadows, shadowWeight, highlights, highlightWeight);
    green = toneChannel(green, shadows, shadowWeight, highlights, highlightWeight);
    blue = toneChannel(blue, shadows, shadowWeight, highlights, highlightWeight);
    red = toneChannel(red, blacks, blackWeight, whites, whiteWeight);
    green = toneChannel(green, blacks, blackWeight, whites, whiteWeight);
    blue = toneChannel(blue, blacks, blackWeight, whites, whiteWeight);
    const colorLuminance = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    const chroma = Math.max(red, green, blue) - Math.min(red, green, blue);
    const colorStrength = saturation * (1 + vibrance * (1 - chroma));
    red = clamp(colorLuminance + (red - colorLuminance) * colorStrength + warmth + tint);
    green = clamp(colorLuminance + (green - colorLuminance) * colorStrength - tint);
    blue = clamp(colorLuminance + (blue - colorLuminance) * colorStrength - warmth + tint);
    if (softLight) {
      red += softLight * red * (1 - red);
      green += softLight * green * (1 - green);
      blue += softLight * blue * (1 - blue);
    }
    if (adjustments.vignette !== 0) {
      const x = (offset / 4) % width;
      const y = Math.floor(offset / 4 / width);
      const distance = Math.hypot(
        (x + 0.5 - imageRect.left - imageRect.width / 2) / (imageRect.width / 2),
        (y + 0.5 - imageRect.top - imageRect.height / 2) / (imageRect.height / 2),
      );
      const edge = smoothstep(0.2, 1.1, distance) * Math.abs(adjustments.vignette / 100) * 0.75;
      if (adjustments.vignette > 0) {
        red *= 1 - edge;
        green *= 1 - edge;
        blue *= 1 - edge;
      } else {
        red += (1 - red) * edge;
        green += (1 - green) * edge;
        blue += (1 - blue) * edge;
      }
    }

    output[offset] = clamp(red) * BYTE_MAX;
    output[offset + 1] = clamp(green) * BYTE_MAX;
    output[offset + 2] = clamp(blue) * BYTE_MAX;
  }
  return finishSpatialAdjustments(output, width, height, adjustments, imageRect);
}

function finishSpatialAdjustments(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
  adjustments: ImageAdjustments,
  imageRect: { width: number; height: number },
) {
  const detailed = applyImageDetail(pixels, width, height, adjustments.sharpness, adjustments.clarity, adjustments.grain);
  return applyImageGlow(detailed, width, height, adjustments.glow, imageRect);
}

function contrastChannel(value: number, contrast: number) {
  return clamp((value - 0.5) * contrast + 0.5);
}

function toneChannel(
  value: number,
  lowerAmount: number,
  lowerWeight: number,
  upperAmount: number,
  upperWeight: number,
) {
  return adjustRange(adjustRange(value, lowerAmount, lowerWeight), upperAmount, upperWeight);
}

function adjustRange(value: number, amount: number, weight: number) {
  const room = amount >= 0 ? 1 - value : value;
  return clamp(value + room * amount * weight);
}

function smoothstep(edge0: number, edge1: number, value: number) {
  const position = clamp((value - edge0) / (edge1 - edge0));
  return position * position * (3 - 2 * position);
}

function clamp(value: number) {
  return Math.max(0, Math.min(1, value));
}
