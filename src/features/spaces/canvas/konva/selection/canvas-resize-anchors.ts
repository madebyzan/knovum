import { isNil } from "es-toolkit";

import type { SpaceNodeResizeMode } from "../../node-size";

export const CANVAS_PROPORTIONAL_RESIZE_ANCHORS = [
  "top-left",
  "top-right",
  "bottom-left",
  "bottom-right",
] as const;

export const CANVAS_FREE_RESIZE_ANCHORS = [
  "top-left",
  "top-center",
  "top-right",
  "middle-right",
  "bottom-right",
  "bottom-center",
  "bottom-left",
  "middle-left",
] as const;

export const CANVAS_TEXT_RESIZE_ANCHORS = ["middle-right", "bottom-right", "middle-left"] as const;
const PROPORTIONAL_ANCHORS = new Set<string>(CANVAS_PROPORTIONAL_RESIZE_ANCHORS);

export function canvasResizeAnchors(mode: SpaceNodeResizeMode) {
  if (mode === "fixed") return [];
  if (mode === "proportional") return [...CANVAS_PROPORTIONAL_RESIZE_ANCHORS];
  if (mode === "width-and-proportional") return [...CANVAS_TEXT_RESIZE_ANCHORS];
  return [...CANVAS_FREE_RESIZE_ANCHORS];
}

export function isProportionalResizeAnchor(mode: SpaceNodeResizeMode, anchor: string | null | undefined) {
  return (
    mode === "proportional" ||
    (mode === "width-and-proportional" && !isNil(anchor) && PROPORTIONAL_ANCHORS.has(anchor))
  );
}
