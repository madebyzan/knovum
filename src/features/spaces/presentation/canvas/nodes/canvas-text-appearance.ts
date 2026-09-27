import { DEFAULT_CANVAS_TEXT_FONT_SIZE } from "../../../canvas/node-size";
import type { CanvasNodeAppearance } from "../../../canvas/schema/document";

const FILL_COLORS = {
  yellow: "var(--knovum-space-note-bg-yellow)",
  green: "var(--knovum-space-note-bg-green)",
  blue: "var(--knovum-space-note-bg-blue)",
  pink: "var(--knovum-space-note-bg-pink)",
  gray: "var(--knovum-space-note-bg-gray)",
} as const;
const TEXT_COLORS = {
  ink: "var(--knovum-space-text-fg-default)",
  muted: "var(--knovum-space-text-fg-subtle)",
  red: "var(--knovum-space-text-fg-red)",
  blue: "var(--knovum-space-text-fg-blue)",
  green: "var(--knovum-space-text-fg-green)",
  purple: "var(--knovum-space-text-fg-purple)",
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
  return kind === "note" ? "var(--knovum-space-note-fg)" : TEXT_COLORS[appearance?.textColor ?? "ink"];
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
