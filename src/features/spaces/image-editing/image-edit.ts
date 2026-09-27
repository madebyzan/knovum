import { clamp } from "es-toolkit";

import type { SpaceImagePayload } from "../model/nodes/node-payload";

export type ImageEdit = NonNullable<SpaceImagePayload["imageEdit"]>;
export type ImageAdjustments = ImageEdit["adjustments"];
export type ImageCrop = ImageEdit["crop"];

export function defaultImageEdit(): ImageEdit {
  return {
    version: 5,
    crop: { x: 0, y: 0, width: 1, height: 1 },
    rotation: 0,
    flipHorizontal: false,
    flipVertical: false,
    adjustments: {
      exposure: 0,
      contrast: 0,
      highlights: 0,
      shadows: 0,
      whites: 0,
      blacks: 0,
      saturation: 0,
      warmth: 0,
      sharpness: 0,
      vignette: 0,
      vibrance: 0,
      tint: 0,
      clarity: 0,
      grain: 0,
      softLight: 0,
      glow: 0,
    },
    eraserStrokes: [],
  };
}

export function imageEditOf(data: SpaceImagePayload): ImageEdit {
  return data.imageEdit ?? defaultImageEdit();
}

export function centeredCrop(sourceAspect: number, targetAspect: number): ImageCrop {
  if (sourceAspect > targetAspect) {
    const width = targetAspect / sourceAspect;
    return { x: (1 - width) / 2, y: 0, width, height: 1 };
  }
  const height = sourceAspect / targetAspect;
  return { x: 0, y: (1 - height) / 2, width: 1, height };
}

export function resizeCrop(crop: ImageCrop, width: number, height: number): ImageCrop {
  const nextWidth = clamp(width, 0.01, 1);
  const nextHeight = clamp(height, 0.01, 1);
  return {
    x: clamp(crop.x + (crop.width - nextWidth) / 2, 0, 1 - nextWidth),
    y: clamp(crop.y + (crop.height - nextHeight) / 2, 0, 1 - nextHeight),
    width: nextWidth,
    height: nextHeight,
  };
}

export function scaleCropToPixelSize(input: {
  crop: ImageCrop;
  sourceWidth: number;
  sourceHeight: number;
  axis: "width" | "height";
  pixels: number;
}): ImageCrop {
  const { crop, sourceWidth, sourceHeight, axis, pixels } = input;
  const current = axis === "width" ? crop.width * sourceWidth : crop.height * sourceHeight;
  const factor = Math.max(
    Math.max(0.01 / crop.width, 0.01 / crop.height),
    Math.min(Math.min(1 / crop.width, 1 / crop.height), pixels / current),
  );
  return resizeCrop(crop, crop.width * factor, crop.height * factor);
}

export function nextQuarterTurn(rotation: ImageEdit["rotation"]): ImageEdit["rotation"] {
  return (rotation + 90) % 360;
}
