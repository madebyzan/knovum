export interface DiagramSize {
  readonly width: number;
  readonly height: number;
}

export interface MermaidViewerLabels {
  readonly zoomIn: string;
  readonly zoomOut: string;
  readonly preview: string;
  readonly close: string;
}

export interface MermaidPanZoomState {
  readonly scale: number;
  readonly x: number;
  readonly y: number;
}

export const MERMAID_MIN_INLINE_SCALE = 0.78;
const MERMAID_MIN_ZOOM = 0.25;
const MERMAID_MAX_ZOOM = 4;
export const MERMAID_ZOOM_STEP = 0.25;

export const DEFAULT_MERMAID_LABELS: MermaidViewerLabels = {
  zoomIn: "Zoom in",
  zoomOut: "Zoom out",
  preview: "View fullscreen",
  close: "Close",
};

const FALLBACK_SIZE: DiagramSize = { width: 640, height: 320 };

function parsePositiveNumber(raw: string | null): number | null {
  if (!raw || raw.includes("%")) return null;
  const value = Number.parseFloat(raw);
  return Number.isFinite(value) && value > 0 ? value : null;
}

/** Extracts the intrinsic coordinate dimensions of a rendered Mermaid SVG element. */
export function parseSvgDimensions(svg: Element): DiagramSize {
  const viewBox = svg.getAttribute("viewBox");
  if (viewBox) {
    const parts = viewBox
      .trim()
      .split(/[\s,]+/u)
      .map(Number);
    const width = parts[2];
    const height = parts[3];
    if (width && height && Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0) {
      return { width, height };
    }
  }
  const width = parsePositiveNumber(svg.getAttribute("width"));
  const height = parsePositiveNumber(svg.getAttribute("height"));
  if (width && height) return { width, height };
  return FALLBACK_SIZE;
}

export function clampMermaidZoom(scale: number): number {
  const bounded = Math.min(MERMAID_MAX_ZOOM, Math.max(MERMAID_MIN_ZOOM, scale));
  return Math.round(bounded * 100) / 100;
}

/**
 * Computes the readable inline display scale for a Mermaid diagram:
 * narrow diagrams stay at 100%, moderately wide diagrams fit cleanly,
 * and dense wide diagrams never shrink below `MERMAID_MIN_INLINE_SCALE`.
 */
export function computeInlineMermaidScale(diagram: DiagramSize, viewportWidth: number): number {
  if (diagram.width <= 0 || viewportWidth <= 0) return 1;
  if (diagram.width <= viewportWidth) return 1;
  return Math.max(MERMAID_MIN_INLINE_SCALE, clampMermaidZoom(viewportWidth / diagram.width));
}

/** Computes the initial fullscreen lightbox scale that frames the whole diagram cleanly. */
export function computeLightboxFitScale(diagram: DiagramSize, viewport: DiagramSize): number {
  if (diagram.width <= 0 || diagram.height <= 0) return 1;
  const usableWidth = Math.max(160, viewport.width - 112);
  const usableHeight = Math.max(160, viewport.height - 144);
  return clampMermaidZoom(Math.min(usableWidth / diagram.width, usableHeight / diagram.height, 1.5));
}

/** Adjusts pan offset when zooming so the point under the cursor stays visually anchored. */
export function zoomAroundAnchor(
  current: MermaidPanZoomState,
  nextScale: number,
  anchorOffset: { readonly x: number; readonly y: number },
): MermaidPanZoomState {
  const scale = clampMermaidZoom(nextScale);
  if (scale === current.scale || current.scale <= 0) return current;
  const ratio = scale / current.scale;
  return {
    scale,
    x: anchorOffset.x - (anchorOffset.x - current.x) * ratio,
    y: anchorOffset.y - (anchorOffset.y - current.y) * ratio,
  };
}
