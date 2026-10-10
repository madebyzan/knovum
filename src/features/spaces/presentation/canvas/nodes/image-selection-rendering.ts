import type { DeepReadonly } from "vue";

import type { ImageSelection } from "../../../contracts/generated/ImageSelection";
import type { ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";

function drawMask(context: CanvasRenderingContext2D, mask: HTMLImageElement) {
  const { width, height } = context.canvas;
  context.drawImage(mask, 0, 0, width, height);
  const pixels = context.getImageData(0, 0, width, height);
  for (let i = 0; i < pixels.data.length; i += 4) {
    const selected = pixels.data[i]!;
    pixels.data[i] = 45;
    pixels.data[i + 1] = 111;
    pixels.data[i + 2] = 239;
    pixels.data[i + 3] = selected * 0.4;
  }
  context.putImageData(pixels, 0, 0);
}

export function drawImageSelection(
  context: CanvasRenderingContext2D,
  mask: HTMLImageElement | undefined,
  selection: DeepReadonly<ImageSelection> | undefined,
  size: ImageCanvasSize,
) {
  if (mask) drawMask(context, mask);
  if (!selection) return;
  context.save();
  context.scale(size.pixelRatio, size.pixelRatio);
  context.lineWidth = 1.5;
  for (const point of selection.points) {
    context.beginPath();
    context.arc(point.x * size.width, point.y * size.height, 4, 0, 2 * Math.PI);
    context.fillStyle = point.foreground ? "#2d6fef" : "#f14b5d";
    context.fill();
    context.strokeStyle = "white";
    context.stroke();
  }
  const box = selection.boxRect;
  context.strokeStyle = "#2d6fef";
  if (box)
    context.strokeRect(
      box.x * size.width,
      box.y * size.height,
      box.width * size.width,
      box.height * size.height,
    );
  context.restore();
}
