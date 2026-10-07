type CanvasPreviewLevel = "compact" | "summary" | "full";

export interface CanvasPreviewPolicy {
  level: CanvasPreviewLevel;
  previewItemLimit: 2 | 4 | 10;
}

export interface CanvasNodePresentationContext {
  viewport: CanvasPreviewPolicy;
  mode: "interactive" | "preview";
}

const POLICIES: Readonly<Record<CanvasPreviewLevel, CanvasPreviewPolicy>> = {
  compact: { level: "compact", previewItemLimit: 2 },
  summary: { level: "summary", previewItemLimit: 4 },
  full: { level: "full", previewItemLimit: 10 },
};

export function canvasPreviewPolicy(zoomPercent: number, nodeCount: number): CanvasPreviewPolicy {
  const full = richPreviewZoomThreshold(nodeCount);
  if (zoomPercent >= full) return POLICIES.full;
  if (zoomPercent >= Math.max(38, full - 28)) return POLICIES.summary;
  return POLICIES.compact;
}

export function canvasNodePresentationContext(
  viewport: CanvasPreviewPolicy,
  interactive: boolean,
): CanvasNodePresentationContext {
  return { viewport, mode: interactive ? "interactive" : "preview" };
}

export function shouldRenderRichCanvasPreview(zoomPercent: number, nodeCount: number) {
  return canvasPreviewPolicy(zoomPercent, nodeCount).level === "full";
}

export function richPreviewZoomThreshold(nodeCount: number) {
  if (nodeCount >= 1_500) return 115;
  if (nodeCount >= 800) return 100;
  if (nodeCount >= 400) return 88;
  return 78;
}
