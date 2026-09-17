import type { CanvasNodeAppearance } from "../../../canvas/contract/canvas-document";
import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../../canvas/node-size";

const FILL_COLORS = {
  yellow: "var(--knovum-canvas-note-yellow)",
  green: "var(--knovum-canvas-note-green)",
  blue: "var(--knovum-canvas-note-blue)",
  pink: "var(--knovum-canvas-note-pink)",
  gray: "var(--knovum-canvas-note-gray)",
} as const;
const TEXT_COLORS = {
  ink: "var(--knovum-canvas-text-ink)",
  muted: "var(--knovum-canvas-text-muted)",
  red: "var(--knovum-canvas-text-red)",
  blue: "var(--knovum-canvas-text-blue)",
  green: "var(--knovum-canvas-text-green)",
  purple: "var(--knovum-canvas-text-purple)",
} as const;

export function canvasTextAppearanceClasses(
  kind: "note" | "text",
  appearance: CanvasNodeAppearance | undefined,
) {
  return [
    appearance?.fontWeight === "bold" ? "font-bold" : undefined,
    appearance?.fontStyle === "italic" ? "italic" : undefined,
    appearance?.textDecoration === "underline" ? "underline" : undefined,
    appearance?.textAlign === "left" || (!appearance?.textAlign && kind === "note") ? "text-left" : undefined,
    appearance?.textAlign === "center" || (!appearance?.textAlign && kind === "text")
      ? "text-center"
      : undefined,
    appearance?.textAlign === "right" ? "text-right" : undefined,
  ];
}

export function canvasTextAppearanceStyle(
  kind: "note" | "text",
  appearance: CanvasNodeAppearance | undefined,
) {
  return {
    fontSize: `${appearance?.fontSize ?? DEFAULT_CANVAS_TEXT_FONT_SIZE}pt`,
    color: kind === "note" ? "var(--knovum-canvas-note-ink)" : TEXT_COLORS[appearance?.textColor ?? "ink"],
    ...(kind === "note"
      ? { backgroundColor: appearance?.customFill ?? FILL_COLORS[appearance?.fill ?? "yellow"] }
      : {}),
  };
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
