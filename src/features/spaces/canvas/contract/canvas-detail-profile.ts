export type CanvasDetailLevel = "compact" | "summary" | "full";

export interface CanvasViewportDetailProfile {
  level: CanvasDetailLevel;
  previewItemLimit: 2 | 4 | 10;
}

export interface CanvasNodePresentationContext {
  viewport: CanvasViewportDetailProfile;
  mode: "interactive" | "preview";
}

const PROFILES: Readonly<Record<CanvasDetailLevel, CanvasViewportDetailProfile>> = {
  compact: { level: "compact", previewItemLimit: 2 },
  summary: { level: "summary", previewItemLimit: 4 },
  full: { level: "full", previewItemLimit: 10 },
};

export function canvasViewportDetailProfile(
  zoomPercent: number,
  nodeCount: number,
): CanvasViewportDetailProfile {
  const full = richPreviewZoomThreshold(nodeCount);
  if (zoomPercent >= full) return PROFILES.full;
  if (zoomPercent >= Math.max(38, full - 28)) return PROFILES.summary;
  return PROFILES.compact;
}

export function canvasNodePresentationContext(
  viewport: CanvasViewportDetailProfile,
  interactive: boolean,
): CanvasNodePresentationContext {
  return { viewport, mode: interactive ? "interactive" : "preview" };
}

export function shouldRenderRichCanvasPreview(zoomPercent: number, nodeCount: number) {
  return canvasViewportDetailProfile(zoomPercent, nodeCount).level === "full";
}

export function richPreviewZoomThreshold(nodeCount: number) {
  if (nodeCount >= 1_500) return 115;
  if (nodeCount >= 800) return 100;
  if (nodeCount >= 400) return 88;
  return 78;
}
