import { rotatedImageSize, type ImageEdit, type ImagePixelRect } from "./image-edit";
import { applyImageTone } from "./image-tone";

export interface ImageCanvasSize {
  width: number;
  height: number;
  pixelRatio: number;
}

export function displayedImageRect(
  size: ImageCanvasSize,
  naturalSize: { width: number; height: number },
  edit: ImageEdit,
) {
  if (!naturalSize.width || !naturalSize.height) return undefined;
  const rendered = rotatedImageSize(
    naturalSize.width * edit.crop.width,
    naturalSize.height * edit.crop.height,
    edit.rotation,
  );
  const scale = Math.min(size.width / rendered.width, size.height / rendered.height);
  const width = rendered.width * scale;
  const height = rendered.height * scale;
  return { left: (size.width - width) / 2, top: (size.height - height) / 2, width, height };
}

export function renderEditedImage(
  context: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D,
  image: HTMLImageElement | ImageBitmap,
  size: ImageCanvasSize,
  edit: ImageEdit,
) {
  const width = size.width * size.pixelRatio;
  const height = size.height * size.pixelRatio;
  context.clearRect(0, 0, width, height);
  context.save();
  const crop = edit.crop;
  const naturalWidth = "naturalWidth" in image ? image.naturalWidth : image.width;
  const naturalHeight = "naturalHeight" in image ? image.naturalHeight : image.height;
  const sourceWidth = naturalWidth * crop.width;
  const sourceHeight = naturalHeight * crop.height;
  const { width: rotatedWidth, height: rotatedHeight } = rotatedImageSize(
    sourceWidth,
    sourceHeight,
    edit.rotation,
  );
  const scale = Math.min(width / rotatedWidth, height / rotatedHeight);
  context.translate(width / 2, height / 2);
  context.scale(edit.flipHorizontal ? -scale : scale, edit.flipVertical ? -scale : scale);
  context.rotate((edit.rotation * Math.PI) / 180);
  context.drawImage(
    image,
    naturalWidth * crop.x,
    naturalHeight * crop.y,
    sourceWidth,
    sourceHeight,
    -sourceWidth / 2,
    -sourceHeight / 2,
    sourceWidth,
    sourceHeight,
  );
  context.restore();
  renderToneAdjustments(context, edit, {
    left: (width - rotatedWidth * scale) / 2,
    top: (height - rotatedHeight * scale) / 2,
    width: rotatedWidth * scale,
    height: rotatedHeight * scale,
  });
}

function renderToneAdjustments(
  context: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D,
  edit: ImageEdit,
  imageRect: ImagePixelRect,
) {
  if (Object.values(edit.adjustments).every((value) => value === 0)) return;
  const { width, height } = context.canvas;
  const frame = context.getImageData(0, 0, width, height);
  const pixels = applyImageTone(frame.data, edit.adjustments, { width, height, imageRect });
  context.putImageData(new ImageData(new Uint8ClampedArray(pixels), width, height), 0, 0);
}

export function paintImageBitmap(context: CanvasRenderingContext2D, bitmap: ImageBitmap) {
  const canvas = context.canvas;
  if (canvas.width !== bitmap.width) canvas.width = bitmap.width;
  if (canvas.height !== bitmap.height) canvas.height = bitmap.height;
  context.clearRect(0, 0, bitmap.width, bitmap.height);
  context.drawImage(bitmap, 0, 0);
}
