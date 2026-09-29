import { clamp, compact, uniq } from "es-toolkit";

import {
  fitsChrome,
  type PlacementInput,
  type SurfacePlacement,
  type SurfaceRect,
  type SurfaceSide,
} from "./selection-surface-geometry";

function freeVerticalBands(left: number, right: number, safe: SurfaceRect, reserved: readonly SurfaceRect[]) {
  let bands = [{ top: safe.top, bottom: safe.bottom }];
  for (const area of reserved) {
    if (right <= area.left || left >= area.right) continue;
    bands = bands.flatMap((band) => {
      if (area.bottom <= band.top || area.top >= band.bottom) return [band];
      return [
        ...(area.top > band.top ? [{ top: band.top, bottom: area.top }] : []),
        ...(area.bottom < band.bottom ? [{ top: area.bottom, bottom: band.bottom }] : []),
      ];
    });
  }
  return bands;
}

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
  const bands = freeVerticalBands(left, left + width, safe, input.reserved);
  const center = (visible.top + visible.bottom) / 2;
  const overlap = (band: { top: number; bottom: number }) =>
    Math.max(0, Math.min(band.bottom, visible.bottom) - Math.max(band.top, visible.top));
  const best = bands
    .map((band) => ({ ...band, shown: Math.min(height, band.bottom - band.top) }))
    .filter((band) => band.shown >= Math.min(160, height))
    .toSorted(
      (a, b) =>
        overlap(b) - overlap(a) ||
        b.shown - a.shown ||
        Math.abs(center - clamp(center, a.top + a.shown / 2, a.bottom - a.shown / 2)) -
          Math.abs(center - clamp(center, b.top + b.shown / 2, b.bottom - b.shown / 2)),
    )[0];
  if (!best) return undefined;
  const top = clamp(center - best.shown / 2, best.top, best.bottom - best.shown);
  return { left, top, maxHeight: best.shown, side };
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

/** Keep an editor beside the selected object and scroll it within the available vertical band. */
export function placeInspectorSurface(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
): SurfacePlacement | undefined {
  const height = Math.min(input.surface.height, safe.bottom - safe.top);
  if (input.surface.width > safe.right - safe.left || height <= 0) return undefined;
  const previousHorizontal =
    input.previousSide === "left" || input.previousSide === "right" ? input.previousSide : undefined;
  const horizontalSides = uniq(compact([previousHorizontal, "right", "left"]));
  for (const side of horizontalSides) {
    const candidate = candidateForSide(input, safe, visible, { side, height });
    if (candidate) return candidate;
  }
  const previousVertical =
    input.previousSide === "above" || input.previousSide === "below" ? input.previousSide : undefined;
  const verticalSides = uniq(compact([previousVertical, "above", "below"]));
  for (const side of verticalSides) {
    const candidate = candidateForSide(input, safe, visible, { side, height });
    if (candidate) return candidate;
  }
  return inside(input, safe, visible, height);
}
