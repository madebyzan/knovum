import type { CanvasBackdrop } from "../../shared/contracts/generated/CanvasBackdrop";
import type { CanvasBackgroundPattern } from "../../shared/contracts/generated/CanvasBackgroundPattern";
import { canvasBackgroundPatterns } from "../../shared/contracts/generated/contract-values";

export const CANVAS_PATTERN_LABEL_KEY: Readonly<Record<CanvasBackgroundPattern, string>> = {
  dots: "settings.appearance.canvas.patterns.dots",
  grid: "settings.appearance.canvas.patterns.grid",
  none: "settings.appearance.canvas.patterns.none",
};

export const CANVAS_BACKDROP_LABEL_KEY: Readonly<Record<CanvasBackdrop, string>> = {
  none: "settings.appearance.canvas.backdrops.none",
  "morning-mist": "settings.appearance.canvas.backdrops.morningMist",
  "summer-glow": "settings.appearance.canvas.backdrops.summerGlow",
  "aurora-silk": "settings.appearance.canvas.backdrops.auroraSilk",
  "quiet-canopy": "settings.appearance.canvas.backdrops.quietCanopy",
  "dusk-tide": "settings.appearance.canvas.backdrops.duskTide",
};

export function nextCanvasBackgroundPattern(current: CanvasBackgroundPattern): CanvasBackgroundPattern {
  const index = canvasBackgroundPatterns.indexOf(current);
  return canvasBackgroundPatterns[(index + 1) % canvasBackgroundPatterns.length] ?? "dots";
}
