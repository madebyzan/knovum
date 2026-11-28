import type Konva from "konva";

import type { VideoOutput, VideoTextOverlay } from "../project";

/** ASS script height the export uses; text sizes stay 720p-relative in both paths. */
const SCRIPT_HEIGHT = 720;
const LINE_HEIGHT = 1.28;
/** Safe horizontal text region (86% of frame width) matching ASS 7% left/right margins. */
export const TEXT_SAFE_WIDTH_RATIO = 0.86;

function textShadowConfig(
  style: VideoTextOverlay["style"],
  scale: number,
  strength: number,
  outlined: boolean,
) {
  if (style.decoration === "shadow") {
    return {
      shadowColor: style.decorationColor,
      shadowOffset: { x: strength * 0.7, y: strength * 0.9 },
      shadowBlur: Math.max(2, strength * 1.4),
      shadowOpacity: 0.85,
      shadowEnabled: true,
    };
  }
  if (outlined) {
    return {
      shadowColor: "#000000",
      shadowOffset: { x: 0, y: Math.max(1, 1.5 * scale) },
      shadowBlur: Math.max(2, 3 * scale),
      shadowOpacity: 0.45,
      shadowEnabled: true,
    };
  }
  return {
    shadowColor: "#000000",
    shadowOffset: { x: 0, y: 0 },
    shadowBlur: 0,
    shadowOpacity: 0,
    shadowEnabled: false,
  };
}

/**
 * Konva description of one overlay. The mapping mirrors the broadcast-grade ASS style the export
 * writes: safe-width wrapping, middle vertical anchor, soft contrast shadow on outlines, and
 * translucent rounded pill box decoration.
 */
export function textConfig(item: VideoTextOverlay, output: VideoOutput): Konva.TextConfig {
  const style = item.style;
  const scale = output.height / SCRIPT_HEIGHT;
  const strength = style.decorationStrength * scale;
  const outlined = style.decoration === "outline" && strength > 0;
  return {
    text: item.text,
    fontFamily: `"${style.family}", sans-serif`,
    fontStyle: `${style.italic ? "italic " : ""}${style.weight}`,
    fontSize: style.size * scale,
    lineHeight: LINE_HEIGHT,
    align: style.align,
    fill: style.color,
    x: item.x * output.width,
    y: item.y * output.height,
    stroke: outlined ? style.decorationColor : style.color,
    strokeWidth: outlined ? strength * 2 : 0,
    lineJoin: "round",
    fillAfterStrokeEnabled: true,
    ...textShadowConfig(style, scale, strength, outlined),
    wrap: "word",
    perfectDrawEnabled: false,
  };
}

/** Horizontal anchor of the text box, matching the canvas and ASS alignment the export uses. */
export function textAnchorX(align: VideoTextOverlay["style"]["align"], width: number) {
  if (align === "center") return width / 2;
  return align === "right" ? width : 0;
}

export function textBoxMetrics(item: VideoTextOverlay, output: VideoOutput) {
  const scale = output.height / SCRIPT_HEIGHT;
  if (item.style.decoration !== "box") {
    return { paddingX: 0, paddingY: 0, cornerRadius: 0 };
  }
  const base = Math.max(2, item.style.decorationStrength) * scale;
  return {
    paddingX: base * 2.1,
    paddingY: base * 0.85,
    cornerRadius: Math.round(6 * scale),
  };
}
