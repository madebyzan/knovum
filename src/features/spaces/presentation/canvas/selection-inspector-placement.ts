import { clamp, compact, uniq } from "es-toolkit";

import {
  fitsChrome,
  type PlacementInput,
  type SurfacePlacement,
  type SurfaceRect,
  type SurfaceSide,
} from "./selection-surface-geometry";

function horizontal(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
  options: { side: "left" | "right"; height: number },
): SurfacePlacement | undefined {
  const { side, height } = options;
  const width = input.surface.width;
  const left =
    side === "right" ? input.selection.right + input.gap : input.selection.left - input.gap - width;
  if (left < safe.left || left + width > safe.right) return undefined;
  const centered = clamp((visible.top + visible.bottom - height) / 2, safe.top, safe.bottom - height);
  for (const top of [centered, clamp(visible.top, safe.top, safe.bottom - height)]) {
    if (fitsChrome({ left, top, right: left + width, bottom: top + height }, input.reserved))
      return { left, top, maxHeight: height, side };
  }
  return undefined;
}

function vertical(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
  options: { side: "above" | "below"; height: number },
): SurfacePlacement | undefined {
  const { side, height } = options;
  const space =
    side === "above"
      ? input.selection.top - input.gap - safe.top
      : safe.bottom - input.selection.bottom - input.gap;
  if (space < Math.min(160, height)) return undefined;
  const shown = Math.min(height, space);
  const width = input.surface.width;
  const left = clamp((visible.left + visible.right - width) / 2, safe.left, safe.right - width);
  const top = side === "above" ? input.selection.top - input.gap - shown : input.selection.bottom + input.gap;
  return fitsChrome({ left, top, right: left + width, bottom: top + shown }, input.reserved)
    ? { left, top, maxHeight: shown, side }
    : undefined;
}

function inside(input: PlacementInput, safe: SurfaceRect, visible: SurfaceRect, height: number) {
  const width = input.surface.width;
  const left = clamp((visible.left + visible.right - width) / 2, safe.left, safe.right - width);
  const top = clamp(visible.top + 8, safe.top, safe.bottom - height);
  return fitsChrome({ left, top, right: left + width, bottom: top + height }, input.reserved)
    ? { left, top, maxHeight: height, side: "inside" as const }
    : undefined;
}

function candidateForSide(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
  options: { side: SurfaceSide; height: number },
) {
  const { side, height } = options;
  if (side === "left" || side === "right") return horizontal(input, safe, visible, { side, height });
  if (side === "above" || side === "below") return vertical(input, safe, visible, { side, height });
  return undefined;
}

/** Keep an editor panel on its current side until that side no longer fits. */
export function placeInspectorSurface(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
): SurfacePlacement | undefined {
  const height = Math.min(input.surface.height, safe.bottom - safe.top);
  if (input.surface.width > safe.right - safe.left || height <= 0) return undefined;
  if (input.previousSide === "inside") {
    const current = inside(input, safe, visible, height);
    if (current) return current;
  }
  const preferred: (SurfaceSide | undefined)[] = [input.previousSide, "right", "left", "above", "below"];
  const sides = uniq(compact(preferred));
  for (const side of sides) {
    const candidate = candidateForSide(input, safe, visible, { side, height });
    if (candidate) return candidate;
  }
  return inside(input, safe, visible, height);
}
