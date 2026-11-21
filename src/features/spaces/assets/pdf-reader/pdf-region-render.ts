import type { PDFDocumentProxy } from "pdfjs-dist/types/src/display/api";

import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";

export interface RenderedPdfCrop {
  bytes: Uint8Array;
  width: number;
  height: number;
}

/** Renders a single PDF page thumbnail onto a target canvas at crisp device pixel density. */
export async function renderPdfPageThumbnail(
  doc: PDFDocumentProxy,
  pageNumber: number,
  canvas: HTMLCanvasElement,
  targetWidth: number,
): Promise<void> {
  const page = await doc.getPage(pageNumber);
  const base = page.getViewport({ scale: 1 });
  const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
  const scale = (targetWidth / base.width) * dpr;
  const viewport = page.getViewport({ scale });
  canvas.width = Math.max(1, Math.round(viewport.width));
  canvas.height = Math.max(1, Math.round(viewport.height));
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  await page.render({ canvasContext: ctx, viewport, canvas }).promise;
}

/** Renders a high-DPI cropped PNG for a normalized [0..1] rectangle on a PDF page. */
export async function renderPdfRegionPng(
  doc: PDFDocumentProxy,
  pageNumber: number,
  rect: PdfNormalizedRect,
  rotation = 0,
): Promise<RenderedPdfCrop> {
  const page = await doc.getPage(pageNumber);
  const scale = 2.25;
  const viewport = page.getViewport({ scale, rotation: (page.rotate + rotation) % 360 });
  const fullCanvas = document.createElement("canvas");
  fullCanvas.width = Math.max(1, Math.round(viewport.width));
  fullCanvas.height = Math.max(1, Math.round(viewport.height));
  const fullCtx = fullCanvas.getContext("2d");
  if (!fullCtx) throw new Error("Canvas 2D context is unavailable");
  await page.render({ canvasContext: fullCtx, viewport, canvas: fullCanvas }).promise;

  const sx = Math.max(0, Math.floor(rect.x * fullCanvas.width));
  const sy = Math.max(0, Math.floor(rect.y * fullCanvas.height));
  const sw = Math.max(2, Math.min(fullCanvas.width - sx, Math.ceil(rect.width * fullCanvas.width)));
  const sh = Math.max(2, Math.min(fullCanvas.height - sy, Math.ceil(rect.height * fullCanvas.height)));

  const cropCanvas = document.createElement("canvas");
  cropCanvas.width = sw;
  cropCanvas.height = sh;
  const cropCtx = cropCanvas.getContext("2d");
  if (!cropCtx) throw new Error("Crop canvas 2D context is unavailable");
  cropCtx.drawImage(fullCanvas, sx, sy, sw, sh, 0, 0, sw, sh);

  const blob = await new Promise<Blob | null>((resolve) => cropCanvas.toBlob(resolve, "image/png"));
  if (!blob) throw new Error("Failed to encode PDF region as PNG");
  return { bytes: new Uint8Array(await blob.arrayBuffer()), width: sw, height: sh };
}
