import type { ImageEdit } from "./image-edit";

export interface ImageCanvasSize {
  width: number;
  height: number;
  pixelRatio: number;
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
  context.filter = imageFilter(edit);
  const crop = edit.crop;
  const sourceWidth = image.naturalWidth * crop.width;
  const sourceHeight = image.naturalHeight * crop.height;
  const quarterTurn = edit.rotation === 90 || edit.rotation === 270;
  const rotatedWidth = quarterTurn ? sourceHeight : sourceWidth;
  const rotatedHeight = quarterTurn ? sourceWidth : sourceHeight;
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
}

function imageFilter(edit: ImageEdit) {
  const { brightness, contrast, saturation } = edit.adjustments;
  return `brightness(${100 + brightness}%) contrast(${100 + contrast}%) saturate(${100 + saturation}%)`;
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
  const quarterTurn = edit.rotation === 90 || edit.rotation === 270;
  const scale = Math.min(
    (size.width * size.pixelRatio) / (quarterTurn ? cropHeight : cropWidth),
    (size.height * size.pixelRatio) / (quarterTurn ? cropWidth : cropHeight),
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
  return [
    Math.max(0, Math.min(1, sourceX / image.naturalWidth)),
    Math.max(0, Math.min(1, sourceY / image.naturalHeight)),
  ];
}
