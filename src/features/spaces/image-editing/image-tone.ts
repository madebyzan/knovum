import { clamp } from "es-toolkit";

import type { ImageAdjustments, ImagePixelRect } from "./image-edit";
import { applyImageGlow } from "./image-glow";
import { applyImageTexture } from "./image-texture";

const BYTE_MAX = 255;

interface ImageToneFrame {
  width: number;
  height: number;
  imageRect: ImagePixelRect;
}

export function applyImageTone(
  source: Uint8ClampedArray,
  adjustments: ImageAdjustments,
  size: ImageToneFrame,
): Uint8ClampedArray {
  const { width, imageRect } = size;
  const output = new Uint8ClampedArray(source);
  if (
    Object.entries(adjustments).every(
      ([key, value]) => ["sharpness", "clarity", "grain", "glow"].includes(key) || value === 0,
    )
  ) {
    return finishSpatialAdjustments(output, adjustments, size);
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

    red = adjustRange(adjustRange(red, shadows, shadowWeight), highlights, highlightWeight);
    green = adjustRange(adjustRange(green, shadows, shadowWeight), highlights, highlightWeight);
    blue = adjustRange(adjustRange(blue, shadows, shadowWeight), highlights, highlightWeight);
    red = adjustRange(adjustRange(red, blacks, blackWeight), whites, whiteWeight);
    green = adjustRange(adjustRange(green, blacks, blackWeight), whites, whiteWeight);
    blue = adjustRange(adjustRange(blue, blacks, blackWeight), whites, whiteWeight);
    const colorLuminance = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    const chroma = Math.max(red, green, blue) - Math.min(red, green, blue);
    const colorStrength = saturation * (1 + vibrance * (1 - chroma));
    red = clamp(colorLuminance + (red - colorLuminance) * colorStrength + warmth + tint, 0, 1);
    green = clamp(colorLuminance + (green - colorLuminance) * colorStrength - tint, 0, 1);
    blue = clamp(colorLuminance + (blue - colorLuminance) * colorStrength - warmth + tint, 0, 1);
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

    output[offset] = clamp(red, 0, 1) * BYTE_MAX;
    output[offset + 1] = clamp(green, 0, 1) * BYTE_MAX;
    output[offset + 2] = clamp(blue, 0, 1) * BYTE_MAX;
  }
  return finishSpatialAdjustments(output, adjustments, size);
}

function finishSpatialAdjustments(
  pixels: Uint8ClampedArray,
  adjustments: ImageAdjustments,
  size: ImageToneFrame,
) {
  const texturedPixels = applyImageTexture(pixels, {
    width: size.width,
    height: size.height,
    sharpness: adjustments.sharpness,
    clarity: adjustments.clarity,
    grain: adjustments.grain,
  });
  return applyImageGlow(texturedPixels, {
    width: size.width,
    height: size.height,
    amount: adjustments.glow,
    visibleSize: size.imageRect,
  });
}

function contrastChannel(value: number, contrast: number) {
  return clamp((value - 0.5) * contrast + 0.5, 0, 1);
}

function adjustRange(value: number, amount: number, weight: number) {
  const room = amount >= 0 ? 1 - value : value;
  return clamp(value + room * amount * weight, 0, 1);
}

function smoothstep(edge0: number, edge1: number, value: number) {
  const position = clamp((value - edge0) / (edge1 - edge0), 0, 1);
  return position * position * (3 - 2 * position);
}
