import { clamp } from "es-toolkit";

import { rotatedImageSize, type ImageEdit } from "./image-edit";
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
  context: CanvasRenderingContext2D,
  image: HTMLImageElement,
  size: ImageCanvasSize,
  edit: ImageEdit,
) {
  const width = size.width * size.pixelRatio;
  const height = size.height * size.pixelRatio;
  context.clearRect(0, 0, width, height);
  context.save();
  const crop = edit.crop;
  const sourceWidth = image.naturalWidth * crop.width;
  const sourceHeight = image.naturalHeight * crop.height;
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
    image.naturalWidth * crop.x,
    image.naturalHeight * crop.y,
    sourceWidth,
    sourceHeight,
    -sourceWidth / 2,
    -sourceHeight / 2,
    sourceWidth,
    sourceHeight,
  );
  eraseStrokes(context, image, edit);
  context.restore();
  renderToneAdjustments(context, edit, {
    left: (width - rotatedWidth * scale) / 2,
    top: (height - rotatedHeight * scale) / 2,
    width: rotatedWidth * scale,
    height: rotatedHeight * scale,
  });
}

function renderToneAdjustments(
  context: CanvasRenderingContext2D,
  edit: ImageEdit,
  imageRect: { left: number; top: number; width: number; height: number },
) {
  if (Object.values(edit.adjustments).every((value) => value === 0)) return;
  const { width, height } = context.canvas;
  const frame = context.getImageData(0, 0, width, height);
  const pixels = applyImageTone(frame.data, edit.adjustments, { width, height, imageRect });
  context.putImageData(new ImageData(new Uint8ClampedArray(pixels), width, height), 0, 0);
}

function eraseStrokes(context: CanvasRenderingContext2D, image: HTMLImageElement, edit: ImageEdit) {
  context.globalCompositeOperation = "destination-out";
  context.lineCap = "round";
  context.lineJoin = "round";
  const cropCenterX = image.naturalWidth * (edit.crop.x + edit.crop.width / 2);
  const cropCenterY = image.naturalHeight * (edit.crop.y + edit.crop.height / 2);
  for (const stroke of edit.eraserStrokes) {
    context.lineWidth = stroke.diameter * Math.min(image.naturalWidth, image.naturalHeight);
    context.beginPath();
    if (
      stroke.points.length === 2 ||
      (stroke.points.length === 4 &&
        stroke.points[0] === stroke.points[2] &&
        stroke.points[1] === stroke.points[3])
    ) {
      context.arc(
        stroke.points[0]! * image.naturalWidth - cropCenterX,
        stroke.points[1]! * image.naturalHeight - cropCenterY,
        context.lineWidth / 2,
        0,
        Math.PI * 2,
      );
      context.fill();
      continue;
    }
    for (let index = 0; index < stroke.points.length; index += 2) {
      const x = stroke.points[index]! * image.naturalWidth - cropCenterX;
      const y = stroke.points[index + 1]! * image.naturalHeight - cropCenterY;
      if (index === 0) context.moveTo(x, y);
      else context.lineTo(x, y);
    }
    context.stroke();
  }
}

export function canvasPointToImage(
  point: { x: number; y: number },
  image: HTMLImageElement,
  size: ImageCanvasSize,
  edit: ImageEdit,
) {
  const cropWidth = image.naturalWidth * edit.crop.width;
  const cropHeight = image.naturalHeight * edit.crop.height;
  const rotated = rotatedImageSize(cropWidth, cropHeight, edit.rotation);
  const scale = Math.min(
    (size.width * size.pixelRatio) / rotated.width,
    (size.height * size.pixelRatio) / rotated.height,
  );
  const transform = new DOMMatrix()
    .translate((size.width * size.pixelRatio) / 2, (size.height * size.pixelRatio) / 2)
    .scale(edit.flipHorizontal ? -scale : scale, edit.flipVertical ? -scale : scale)
    .rotate(edit.rotation)
    .inverse();
  const local = transform.transformPoint(new DOMPoint(point.x * size.pixelRatio, point.y * size.pixelRatio));
  if (Math.abs(local.x) > cropWidth / 2 || Math.abs(local.y) > cropHeight / 2) return undefined;
  const sourceX = local.x + image.naturalWidth * (edit.crop.x + edit.crop.width / 2);
  const sourceY = local.y + image.naturalHeight * (edit.crop.y + edit.crop.height / 2);
  return [clamp(sourceX / image.naturalWidth, 0, 1), clamp(sourceY / image.naturalHeight, 0, 1)];
}
