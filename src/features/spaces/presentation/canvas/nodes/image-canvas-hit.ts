import { viewportToLocalPoint } from "../../../canvas/viewport-to-local-point";
import type { ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";
import type { ImagePixelRect } from "../../../image-editing/image-edit";

export function imageCanvasHit({
  canvas,
  size,
  rect,
  rotation,
  event,
}: {
  canvas: HTMLCanvasElement | null;
  size: ImageCanvasSize;
  rect: ImagePixelRect | undefined;
  rotation: number;
  event: PointerEvent;
}) {
  if (!canvas || !rect) return;
  const local = viewportToLocalPoint(canvas, event.clientX, event.clientY, rotation);
  const x = (local.x * size.width) / canvas.offsetWidth;
  const y = (local.y * size.height) / canvas.offsetHeight;
  if (x < rect.left || x > rect.left + rect.width || y < rect.top || y > rect.top + rect.height) return;
  return { x, y, point: [(x - rect.left) / rect.width, (y - rect.top) / rect.height] as [number, number] };
}
