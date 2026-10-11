import { clamp } from "es-toolkit";

import type {
  PlacementInput,
  SurfacePlacement,
  SurfaceRect,
  SurfaceSide,
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

function editorBand(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
  options: { left: number; height: number },
) {
  const { left, height } = options;
  const bands = freeVerticalBands(left, left + input.surface.width, safe, input.reserved);
  const center = (visible.top + visible.bottom) / 2;
  const overlap = (band: { top: number; bottom: number }) =>
    Math.max(0, Math.min(band.bottom, visible.bottom) - Math.max(band.top, visible.top));
  return bands
    .map((band) => ({
      top: band.top,
      bottom: band.bottom,
      shown: Math.min(height, band.bottom - band.top),
    }))
    .filter((band) => band.bottom - band.top >= (input.kind === "inspector" ? 160 : Math.min(160, height)))
    .toSorted(
      (a, b) =>
        overlap(b) - overlap(a) ||
        b.shown - a.shown ||
        Math.abs(center - clamp(center, a.top + a.shown / 2, a.bottom - a.shown / 2)) -
          Math.abs(center - clamp(center, b.top + b.shown / 2, b.bottom - b.shown / 2)),
    )[0];
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
  const best = editorBand(input, safe, visible, { left, height });
  if (!best) return undefined;
  const center = (visible.top + visible.bottom) / 2;
  const top = clamp(center - best.shown / 2, best.top, best.bottom - best.shown);
  return { left, top, maxHeight: best.bottom - best.top, side };
}

function vertical(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
  options: { side: "above" | "below"; height: number },
): SurfacePlacement | undefined {
  const { side, height } = options;
  const width = input.surface.width;
  const left = clamp((visible.left + visible.right - width) / 2, safe.left, safe.right - width);
  const edge = side === "above" ? input.selection.top - input.gap : input.selection.bottom + input.gap;
  const band = freeVerticalBands(left, left + width, safe, input.reserved).find(
    (area) => edge >= area.top && edge <= area.bottom,
  );
  if (!band) return undefined;
  const space = side === "above" ? edge - band.top : band.bottom - edge;
  if (space < (input.kind === "inspector" ? 160 : Math.min(160, height))) return undefined;
  const shown = Math.min(height, space);
  const top = side === "above" ? edge - shown : edge;
  return { left, top, maxHeight: space, side };
}

function inside(input: PlacementInput, safe: SurfaceRect, visible: SurfaceRect, height: number) {
  const width = input.surface.width;
  const left = clamp((visible.left + visible.right - width) / 2, safe.left, safe.right - width);
  const band = editorBand(input, safe, visible, { left, height });
  if (!band) return undefined;
  const top = clamp(visible.top + 8, band.top, band.bottom - band.shown);
  return { left, top, maxHeight: band.bottom - top, side: "inside" as const };
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

interface InspectorPlacementOptions {
  safe: SurfaceRect;
  visible: SurfaceRect;
  sides: readonly SurfaceSide[];
  height: number;
}

function firstPlacement(input: PlacementInput, options: InspectorPlacementOptions) {
  for (const side of options.sides) {
    const candidate = candidateForSide(input, options.safe, options.visible, {
      side,
      height: options.height,
    });
    if (candidate) return candidate;
  }
  return undefined;
}

function preferredSides<T extends SurfaceSide>(previous: SurfaceSide | undefined, first: T, second: T): T[] {
  return previous === second ? [second, first] : [first, second];
}

/** Keep an editor beside the selected object and scroll it within the available vertical band. */
export function placeInspectorSurface(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
): SurfacePlacement | undefined {
  const height = Math.min(input.surface.height, safe.bottom - safe.top);
  if (input.surface.width > safe.right - safe.left || height <= 0) return undefined;
  const horizontalSides = preferredSides(input.previousSide, "right", "left");
  const horizontalPlacement = firstPlacement(input, { safe, visible, sides: horizontalSides, height });
  if (horizontalPlacement) return horizontalPlacement;
  const verticalSides = preferredSides(input.previousSide, "above", "below");
  const verticalPlacement = firstPlacement(input, { safe, visible, sides: verticalSides, height });
  if (verticalPlacement) return verticalPlacement;
  return input.kind === "inspector" ? inside(input, safe, visible, height) : undefined;
}
