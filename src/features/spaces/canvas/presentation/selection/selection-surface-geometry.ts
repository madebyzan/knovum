export interface SurfaceRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export type SurfaceSide = "above" | "below" | "left" | "right" | "inside";

export interface PlacementInput {
  selection: SurfaceRect;
  viewport: { width: number; height: number };
  insets: { top: number; right: number; bottom: number; left: number };
  reserved: readonly SurfaceRect[];
  surface: { width: number; height: number };
  kind: "actions" | "compact-editor" | "inspector";
  gap: number;
  previousSide?: SurfaceSide;
  lockSide?: boolean;
}

export interface SurfacePlacement {
  left: number;
  top: number;
  /** Available vertical space, independent of the current editor content height. */
  maxHeight: number;
  side: SurfaceSide;
}

function intersects(a: SurfaceRect, b: SurfaceRect): boolean {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}

export function intersection(a: SurfaceRect, b: SurfaceRect): SurfaceRect | undefined {
  const rect = {
    left: Math.max(a.left, b.left),
    top: Math.max(a.top, b.top),
    right: Math.min(a.right, b.right),
    bottom: Math.min(a.bottom, b.bottom),
  };
  return rect.left <= rect.right && rect.top <= rect.bottom ? rect : undefined;
}

export function fitsChrome(rect: SurfaceRect, reserved: readonly SurfaceRect[]): boolean {
  return reserved.every((area) => !intersects(rect, area));
}
