import type { ImageCrop } from "./image-edit";

export type CropDragHandle = "move" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw";

export interface CropDragOrigin {
  handle: CropDragHandle;
  crop: ImageCrop;
  clientX: number;
  clientY: number;
  viewportWidth: number;
  viewportHeight: number;
  lockedAspect?: number | undefined;
}

export interface CropDrawOrigin {
  x: number;
  y: number;
  viewportWidth: number;
  viewportHeight: number;
  lockedAspect?: number | undefined;
}

const MIN_CROP_SIZE = 0.01;
const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

export function cropFromPointer(origin: CropDragOrigin, clientX: number, clientY: number): ImageCrop {
  const deltaX = (clientX - origin.clientX) / origin.viewportWidth;
  const deltaY = (clientY - origin.clientY) / origin.viewportHeight;
  if (origin.handle === "move") return moveCrop(origin.crop, deltaX, deltaY);
  if (origin.lockedAspect) return lockedCropFromPointer(origin, deltaX, deltaY, origin.lockedAspect);

  let left = origin.crop.x;
  let top = origin.crop.y;
  let right = left + origin.crop.width;
  let bottom = top + origin.crop.height;

  if (origin.handle.includes("w")) left = clamp(left + deltaX, 0, right - MIN_CROP_SIZE);
  if (origin.handle.includes("e")) right = clamp(right + deltaX, left + MIN_CROP_SIZE, 1);
  if (origin.handle.includes("n")) top = clamp(top + deltaY, 0, bottom - MIN_CROP_SIZE);
  if (origin.handle.includes("s")) bottom = clamp(bottom + deltaY, top + MIN_CROP_SIZE, 1);

  return { x: left, y: top, width: right - left, height: bottom - top };
}

export function drawCropFromPointer(
  origin: CropDrawOrigin,
  endX: number,
  endY: number,
): ImageCrop | undefined {
  const x = clamp(origin.x, 0, 1);
  const y = clamp(origin.y, 0, 1);
  const dx = clamp(endX, 0, 1) - x;
  const dy = clamp(endY, 0, 1) - y;
  const directionX = dx < 0 ? -1 : 1;
  const directionY = dy < 0 ? -1 : 1;
  let width = Math.abs(dx);
  let height = Math.abs(dy);
  if (origin.lockedAspect) {
    const ratio = (origin.lockedAspect * origin.viewportHeight) / origin.viewportWidth;
    width = Math.max(width, height * ratio);
    width = Math.min(width, directionX < 0 ? x : 1 - x, (directionY < 0 ? y : 1 - y) * ratio);
    height = width / ratio;
  }
  if (width < MIN_CROP_SIZE || height < MIN_CROP_SIZE) return undefined;
  return {
    x: directionX < 0 ? x - width : x,
    y: directionY < 0 ? y - height : y,
    width,
    height,
  };
}

function lockedCropFromPointer(
  origin: CropDragOrigin,
  deltaX: number,
  deltaY: number,
  aspect: number,
): ImageCrop {
  const handle = origin.handle;
  const ratio = (aspect * origin.viewportHeight) / origin.viewportWidth;
  const west = handle.includes("w");
  const north = handle.includes("n");
  const horizontal = west || handle.includes("e");
  const vertical = north || handle.includes("s");
  const context = {
    crop: origin.crop,
    deltaX,
    deltaY,
    ratio,
    minimumWidth: Math.max(MIN_CROP_SIZE, MIN_CROP_SIZE * ratio),
    west,
    north,
    anchorX: west ? origin.crop.x + origin.crop.width : origin.crop.x,
    anchorY: north ? origin.crop.y + origin.crop.height : origin.crop.y,
  };
  if (horizontal && vertical) return lockedCornerCrop(context);
  return horizontal ? lockedHorizontalCrop(context) : lockedVerticalCrop(context);
}

type LockedCropContext = {
  crop: ImageCrop;
  deltaX: number;
  deltaY: number;
  ratio: number;
  minimumWidth: number;
  west: boolean;
  north: boolean;
  anchorX: number;
  anchorY: number;
};

function lockedCornerCrop(context: LockedCropContext): ImageCrop {
  const { crop, deltaX, deltaY, ratio, minimumWidth, west, north, anchorX, anchorY } = context;
  const widthFromX = crop.width + (west ? -deltaX : deltaX);
  const heightFromY = crop.height + (north ? -deltaY : deltaY);
  const width =
    Math.abs(deltaX / crop.width) >= Math.abs(deltaY / crop.height) ? widthFromX : heightFromY * ratio;
  const maxWidth = Math.min(west ? anchorX : 1 - anchorX, (north ? anchorY : 1 - anchorY) * ratio);
  const nextWidth = clamp(width, minimumWidth, maxWidth);
  const nextHeight = nextWidth / ratio;
  return {
    x: west ? anchorX - nextWidth : anchorX,
    y: north ? anchorY - nextHeight : anchorY,
    width: nextWidth,
    height: nextHeight,
  };
}

function lockedHorizontalCrop(context: LockedCropContext): ImageCrop {
  const { crop, deltaX, ratio, minimumWidth, west, anchorX } = context;
  const centerY = crop.y + crop.height / 2;
  const maxWidth = Math.min(west ? anchorX : 1 - anchorX, 2 * Math.min(centerY, 1 - centerY) * ratio);
  const width = clamp(crop.width + (west ? -deltaX : deltaX), minimumWidth, maxWidth);
  const height = width / ratio;
  return { x: west ? anchorX - width : anchorX, y: centerY - height / 2, width, height };
}

function lockedVerticalCrop(context: LockedCropContext): ImageCrop {
  const { crop, deltaY, ratio, minimumWidth, north, anchorY } = context;
  const centerX = crop.x + crop.width / 2;
  const maxHeight = Math.min(north ? anchorY : 1 - anchorY, (2 * Math.min(centerX, 1 - centerX)) / ratio);
  const height = clamp(crop.height + (north ? -deltaY : deltaY), minimumWidth / ratio, maxHeight);
  const width = height * ratio;
  return { x: centerX - width / 2, y: north ? anchorY - height : anchorY, width, height };
}

function moveCrop(crop: ImageCrop, deltaX: number, deltaY: number): ImageCrop {
  return {
    ...crop,
    x: clamp(crop.x + deltaX, 0, 1 - crop.width),
    y: clamp(crop.y + deltaY, 0, 1 - crop.height),
  };
}
