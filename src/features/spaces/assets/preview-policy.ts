export type SpaceCanvasDetailLevel = "compact" | "summary" | "full";

export function spaceCanvasDetailLevel(zoomPercent: number, nodeCount: number): SpaceCanvasDetailLevel {
  const full = richPreviewZoomThreshold(nodeCount);
  if (zoomPercent >= full) return "full";
  if (zoomPercent >= Math.max(38, full - 28)) return "summary";
  return "compact";
}

export function shouldRenderRichCanvasPreview(zoomPercent: number, nodeCount: number) {
  return spaceCanvasDetailLevel(zoomPercent, nodeCount) === "full";
}

export function richPreviewZoomThreshold(nodeCount: number) {
  if (nodeCount >= 1_500) return 115;
  if (nodeCount >= 800) return 100;
  if (nodeCount >= 400) return 88;
  return 78;
}
