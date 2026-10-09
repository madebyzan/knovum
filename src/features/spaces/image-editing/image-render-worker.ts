/* oxlint-disable unicorn/require-post-message-target-origin -- DedicatedWorkerGlobalScope.postMessage transfers ImageBitmap, not a Window message. */
import { renderEditedImage, type ImageCanvasSize } from "./canvas-image-renderer";
import type { ImageEdit } from "./image-edit";

export interface ImageRenderRequest {
  image: ImageBitmap;
  size: ImageCanvasSize;
  edit: ImageEdit;
}
export type ImageRenderResponse = { bitmap: ImageBitmap } | { error: string };

globalThis.addEventListener("message", (event: MessageEvent<ImageRenderRequest>) => {
  const { image, size, edit } = event.data;
  try {
    const canvas = new OffscreenCanvas(
      Math.max(1, Math.round(size.width * size.pixelRatio)),
      Math.max(1, Math.round(size.height * size.pixelRatio)),
    );
    renderEditedImage(canvas.getContext("2d", { willReadFrequently: true })!, image, size, edit);
    const bitmap = canvas.transferToImageBitmap();
    globalThis.postMessage({ bitmap } satisfies ImageRenderResponse, { transfer: [bitmap] });
  } catch (error) {
    globalThis.postMessage({ error: String(error) } satisfies ImageRenderResponse);
  } finally {
    image.close();
  }
});
