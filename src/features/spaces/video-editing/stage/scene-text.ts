import type Konva from "konva";

import type { VideoOutput, VideoTextOverlay } from "../project";

/** ASS script height the export uses; text sizes stay 720p-relative in both paths. */
const SCRIPT_HEIGHT = 720;
const LINE_HEIGHT = 1.2;

/**
 * Konva description of one overlay. The mapping mirrors the ASS style the export writes: free
 * position, middle vertical anchor and outline, shadow or box decoration.
 */
export function textConfig(item: VideoTextOverlay, output: VideoOutput): Konva.TextConfig {
  const style = item.style;
  const scale = output.height / SCRIPT_HEIGHT;
  const strength = style.decorationStrength * scale;
  const outlined = style.decoration === "outline" && strength > 0;
  const shadowed = style.decoration === "shadow";
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
    shadowColor: shadowed ? style.decorationColor : style.color,
    shadowOffset: { x: shadowed ? strength : 0, y: shadowed ? strength : 0 },
    shadowBlur: 0,
    shadowOpacity: shadowed ? 1 : 0,
    perfectDrawEnabled: false,
  };
}

/** Horizontal anchor of the text box, matching the canvas and ASS alignment the export uses. */
export function textAnchorX(align: VideoTextOverlay["style"]["align"], width: number) {
  if (align === "center") return width / 2;
  return align === "right" ? width : 0;
}

export function textBoxPadding(item: VideoTextOverlay, output: VideoOutput) {
  if (item.style.decoration !== "box") return 0;
  return item.style.decorationStrength * (output.height / SCRIPT_HEIGHT);
}
