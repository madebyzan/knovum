import { clamp } from "es-toolkit";

import {
  fitsChrome,
  type PlacementInput,
  type SurfacePlacement,
  type SurfaceRect,
} from "./selection-surface-geometry";

const CHROME_GAP = 8;
const SIDE_HYSTERESIS = 8;

function atSide(
  input: PlacementInput,
  safe: SurfaceRect,
  position: { anchorX: number; side: "above" | "below" | "inside"; top: number },
): SurfacePlacement | undefined {
  const { width, height } = input.surface;
  const { anchorX, side, top } = position;
  if (top < safe.top || top + height > safe.bottom) return undefined;
  const centered = clamp(anchorX - width / 2, safe.left, safe.right - width);
  const leftLimit = Math.max(safe.left, anchorX - width);
  const rightLimit = Math.min(safe.right - width, anchorX);
  const blocking = input.reserved.filter((area) => top < area.bottom && top + height > area.top);
  const offsets = [
    centered,
    ...blocking.flatMap((area) => [area.left - width - CHROME_GAP, area.right + CHROME_GAP]),
  ];
  const left = offsets
    .filter((value) => value >= leftLimit && value <= rightLimit)
    .toSorted((a, b) => Math.abs(a - centered) - Math.abs(b - centered))
    .find((value) => fitsChrome({ left: value, top, right: value + width, bottom: top + height }, blocking));
  return left === undefined ? undefined : { left, top, maxHeight: height, side };
}

function outside(input: PlacementInput, safe: SurfaceRect, anchorX: number) {
  const aboveTop = input.selection.top - input.gap - input.surface.height;
  const above = () => atSide(input, safe, { anchorX, side: "above", top: aboveTop });
  const below = () =>
    atSide(input, safe, { anchorX, side: "below", top: input.selection.bottom + input.gap });
  if (input.previousSide === "below") {
    const current = below();
    if (current && (input.lockSide || aboveTop < safe.top + SIDE_HYSTERESIS)) return current;
    if (current) return above() ?? current;
  }
  return above() ?? below();
}

/** Keep a short toolbar attached to the selected object, even when other nodes are nearby. */
export function placeActionSurface(
  input: PlacementInput,
  safe: SurfaceRect,
  visible: SurfaceRect,
): SurfacePlacement | undefined {
  const { width, height } = input.surface;
  if (width > safe.right - safe.left || height > safe.bottom - safe.top) return undefined;
  const anchorX = (visible.left + visible.right) / 2;
  const outer = outside(input, safe, anchorX);
  if (outer) return outer;
  return (
    atSide(input, safe, { anchorX, side: "inside", top: visible.top + CHROME_GAP }) ??
    atSide(input, safe, { anchorX, side: "inside", top: visible.bottom - height - CHROME_GAP })
  );
}
