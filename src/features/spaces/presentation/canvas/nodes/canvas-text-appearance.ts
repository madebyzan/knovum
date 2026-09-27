import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../../canvas/node-size";
import type { CanvasNodeAppearance } from "../../../canvas/schema/document";

const FILL_COLORS = {
  yellow: "var(--color-canvas-note-yellow)",
  green: "var(--color-canvas-note-green)",
  blue: "var(--color-canvas-note-blue)",
  pink: "var(--color-canvas-note-pink)",
  gray: "var(--color-canvas-note-gray)",
} as const;
const TEXT_COLORS = {
  ink: "var(--color-canvas-text-ink)",
  muted: "var(--color-canvas-text-muted)",
  red: "var(--color-canvas-text-red)",
  blue: "var(--color-canvas-text-blue)",
  green: "var(--color-canvas-text-green)",
  purple: "var(--color-canvas-text-purple)",
} as const;

export function canvasTextAppearanceClasses(
  kind: "note" | "text",
  appearance: CanvasNodeAppearance | undefined,
) {
  const alignment = appearance?.textAlign ?? (kind === "note" ? "left" : "center");
  return [
    appearance?.fontWeight === "bold" ? "font-bold" : undefined,
    appearance?.fontStyle === "italic" ? "italic" : undefined,
    appearance?.textDecoration === "underline" ? "underline" : undefined,
    { left: "text-left", center: "text-center", right: "text-right" }[alignment],
  ];
}

export function canvasTextAppearanceStyle(
  kind: "note" | "text",
  appearance: CanvasNodeAppearance | undefined,
) {
  const style = {
    fontSize: `${appearance?.fontSize ?? DEFAULT_CANVAS_TEXT_FONT_SIZE}pt`,
    color: canvasTextColor(kind, appearance),
  };
  return Object.assign(style, canvasTextBackground(kind, appearance));
}

function canvasTextColor(kind: "note" | "text", appearance: CanvasNodeAppearance | undefined) {
  return kind === "note" ? "var(--color-canvas-note-ink)" : TEXT_COLORS[appearance?.textColor ?? "ink"];
}

function canvasTextBackground(kind: "note" | "text", appearance: CanvasNodeAppearance | undefined) {
  if (kind !== "note") return {};
  return { backgroundColor: appearance?.customFill ?? FILL_COLORS[appearance?.fill ?? "yellow"] };
}

export function canvasTextVerticalClass(kind: "note" | "text", appearance: CanvasNodeAppearance | undefined) {
  if (appearance?.verticalAlign === "bottom") return "justify-end";
  if (appearance?.verticalAlign === "middle" || (!appearance?.verticalAlign && kind === "text")) {
    return "justify-center";
  }
  return "justify-start";
}

export function canvasTextPresentation(kind: "note" | "text", appearance: CanvasNodeAppearance | undefined) {
  return {
    classes: canvasTextAppearanceClasses(kind, appearance),
    style: canvasTextAppearanceStyle(kind, appearance),
    verticalClass: canvasTextVerticalClass(kind, appearance),
    fontSize: appearance?.fontSize ?? DEFAULT_CANVAS_TEXT_FONT_SIZE,
    textAlign: appearance?.textAlign ?? (kind === "note" ? "left" : "center"),
  } as const;
}
