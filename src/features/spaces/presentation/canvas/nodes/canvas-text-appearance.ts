import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../../canvas/node-size";
import type { CanvasNodeAppearance } from "../../../canvas/schema/document";
import { CONTENT_FILL_COLORS, CONTENT_TEXT_COLORS } from "../canvas-content-colors";

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
  return kind === "note" ? "var(--color-black)" : CONTENT_TEXT_COLORS[appearance?.textColor ?? "ink"];
}

function canvasTextBackground(kind: "note" | "text", appearance: CanvasNodeAppearance | undefined) {
  if (kind !== "note") return {};
  return { backgroundColor: appearance?.customFill ?? CONTENT_FILL_COLORS[appearance?.fill ?? "yellow"] };
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
