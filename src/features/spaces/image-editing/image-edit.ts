import type { SpaceImagePayload } from "../model/nodes/node-payload";

export type ImageEdit = NonNullable<SpaceImagePayload["imageEdit"]>;
export type ImageAdjustments = ImageEdit["adjustments"];
export type ImageCrop = ImageEdit["crop"];

export function defaultImageEdit(): ImageEdit {
  return {
    version: 1,
    crop: { x: 0, y: 0, width: 1, height: 1 },
    rotation: 0,
    flipHorizontal: false,
    flipVertical: false,
    adjustments: { brightness: 0, contrast: 0, saturation: 0 },
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

export function nextQuarterTurn(rotation: ImageEdit["rotation"]): ImageEdit["rotation"] {
  return ((rotation + 90) % 360) as ImageEdit["rotation"];
}
