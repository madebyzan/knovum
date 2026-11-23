import type { CanvasBackgroundPattern } from "../../shared/contracts/generated/CanvasBackgroundPattern";
import { canvasBackgroundPatterns } from "../../shared/contracts/generated/contract-values";

export const CANVAS_PATTERN_LABEL_KEY: Readonly<Record<CanvasBackgroundPattern, string>> = {
  dots: "settings.appearance.canvas.patterns.dots",
  grid: "settings.appearance.canvas.patterns.grid",
  none: "settings.appearance.canvas.patterns.none",
};

export function nextCanvasBackgroundPattern(current: CanvasBackgroundPattern): CanvasBackgroundPattern {
  const index = canvasBackgroundPatterns.indexOf(current);
  return canvasBackgroundPatterns[(index + 1) % canvasBackgroundPatterns.length] ?? "dots";
}
