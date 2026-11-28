import { Group } from "konva/lib/Group";
import { Rect } from "konva/lib/shapes/Rect";
import { Text } from "konva/lib/shapes/Text";

import type { VideoOutput, VideoTextOverlay } from "../project";
import { TEXT_SAFE_WIDTH_RATIO, textAnchorX, textBoxMetrics, textConfig } from "./scene-text";

export interface TextNode {
  group: Group;
  box: Rect;
  label: Text;
  layoutKey?: string | undefined;
}

export function createTextNode(itemId: string): TextNode {
  const group = new Group({ id: itemId });
  const box = new Rect({ listening: false });
  const label = new Text({ name: "video-text", id: `text:${itemId}`, draggable: true });
  group.add(box, label);
  return { group, box, label };
}

function textLayoutKey(item: VideoTextOverlay, output: VideoOutput): string {
  const s = item.style;
  return `${output.width}x${output.height}|${item.x}|${item.y}|${item.text}|${s.family}|${s.weight}|${s.italic}|${s.size}|${s.color}|${s.align}|${s.decoration}|${s.decorationStrength}|${s.decorationColor}`;
}

/**
 * Lays out one overlay inside the safe frame width. Layout results are memoized per node so
 * playback never re-measures unchanged caption glyphs on every animation frame.
 */
export function updateTextNode(node: TextNode, item: VideoTextOverlay, output: VideoOutput) {
  const key = textLayoutKey(item, output);
  if (node.layoutKey === key) return;
  node.layoutKey = key;
  const maxWidth = Math.max(40, Math.round(output.width * TEXT_SAFE_WIDTH_RATIO));
  node.label.setAttr("width", undefined);
  node.label.setAttrs({ ...textConfig(item, output), offsetX: 0, offsetY: 0 });
  if (node.label.width() > maxWidth) {
    node.label.width(maxWidth);
  }
  const width = node.label.width();
  const height = node.label.height();
  const offsetX = textAnchorX(item.style.align, width);
  node.label.setAttrs({ offsetX, offsetY: height / 2 });
  const { paddingX, paddingY, cornerRadius } = textBoxMetrics(item, output);
  node.box.setAttrs({
    x: node.label.x() - offsetX - paddingX,
    y: node.label.y() - height / 2 - paddingY,
    width: width + paddingX * 2,
    height: height + paddingY * 2,
    cornerRadius,
    fill: item.style.decorationColor,
    opacity: 0.78,
    visible: item.style.decoration === "box",
  });
}
