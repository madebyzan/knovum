import type { VideoOutput, VideoTextOverlay } from "../project";

/** ASS script height the export uses; text sizes stay 720p-relative in both paths. */
const SCRIPT_HEIGHT = 720;
const LINE_HEIGHT = 1.2;

export interface TextBlock {
  lines: string[];
  x: number;
  y: number;
  lineHeight: number;
  fontSize: number;
  padding: number;
  width: number;
  height: number;
}

function textBlock(
  context: CanvasRenderingContext2D,
  item: VideoTextOverlay,
  output: VideoOutput,
): TextBlock {
  const scale = output.height / SCRIPT_HEIGHT;
  const fontSize = item.style.size * scale;
  context.font = fontOf(item, fontSize);
  const lines = item.text.replaceAll("\r\n", "\n").split("\n");
  const width = Math.max(...lines.map((line) => context.measureText(line).width));
  const lineHeight = fontSize * LINE_HEIGHT;
  return {
    lines,
    x: item.x * output.width,
    y: item.y * output.height,
    lineHeight,
    fontSize,
    padding: item.style.decoration === "box" ? item.style.decorationStrength * scale : 0,
    width,
    height: lineHeight * lines.length,
  };
}

function fontOf(item: VideoTextOverlay, fontSize: number) {
  const style = item.style;
  return `${style.italic ? "italic " : ""}${style.weight} ${fontSize}px "${style.family}", sans-serif`;
}

/** Mirrors the ASS style set: free position, middle vertical anchor, outline, shadow or box. */
export function drawText(context: CanvasRenderingContext2D, item: VideoTextOverlay, output: VideoOutput) {
  const block = textBlock(context, item, output);
  const scale = output.height / SCRIPT_HEIGHT;
  const strength = item.style.decorationStrength * scale;
  context.save();
  context.font = fontOf(item, block.fontSize);
  context.textAlign = item.style.align;
  context.textBaseline = "middle";
  if (item.style.decoration === "box") paintBox(context, item, block);
  const top = block.y - block.height / 2 + block.lineHeight / 2;
  for (const [index, line] of block.lines.entries()) {
    const baseline = top + index * block.lineHeight;
    if (item.style.decoration === "shadow") {
      context.fillStyle = item.style.decorationColor;
      context.fillText(line, block.x + strength, baseline + strength);
    }
    if (item.style.decoration === "outline" && strength > 0) {
      context.strokeStyle = item.style.decorationColor;
      context.lineWidth = strength * 2;
      context.lineJoin = "round";
      context.miterLimit = 2;
      context.strokeText(line, block.x, baseline);
    }
    context.fillStyle = item.style.color;
    context.fillText(line, block.x, baseline);
  }
  context.restore();
}

function paintBox(context: CanvasRenderingContext2D, item: VideoTextOverlay, block: TextBlock) {
  const left =
    item.style.align === "center"
      ? block.x - block.width / 2
      : item.style.align === "right"
        ? block.x - block.width
        : block.x;
  context.fillStyle = item.style.decorationColor;
  context.fillRect(
    left - block.padding,
    block.y - block.height / 2 - block.padding,
    block.width + block.padding * 2,
    block.height + block.padding * 2,
  );
}

/** Layout rectangle in output pixels, used by the stage for selection and dragging. */
export function textBounds(context: CanvasRenderingContext2D, item: VideoTextOverlay, output: VideoOutput) {
  const block = textBlock(context, item, output);
  const left =
    item.style.align === "center"
      ? block.x - block.width / 2
      : item.style.align === "right"
        ? block.x - block.width
        : block.x;
  return {
    left: left - block.padding,
    top: block.y - block.height / 2 - block.padding,
    width: block.width + block.padding * 2,
    height: block.height + block.padding * 2,
  };
}
