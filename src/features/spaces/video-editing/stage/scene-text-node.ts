import Konva from "konva";

import type { VideoOutput, VideoTextOverlay } from "../project";
import { textAnchorX, textBoxPadding, textConfig } from "./scene-text";

export interface TextNode {
  group: Konva.Group;
  box: Konva.Rect;
  label: Konva.Text;
}

export function createTextNode(itemId: string): TextNode {
  const group = new Konva.Group({ id: itemId });
  const box = new Konva.Rect({ listening: false });
  const label = new Konva.Text({ name: "video-text", id: `text:${itemId}`, draggable: true });
  group.add(box, label);
  return { group, box, label };
}

/**
 * Lays out one overlay. The label is measured by Konva and then anchored like the export's ASS
 * alignment, so the box decoration wraps exactly the glyphs that will be rendered.
 */
export function updateTextNode(node: TextNode, item: VideoTextOverlay, output: VideoOutput) {
  node.label.setAttrs({ ...textConfig(item, output), offsetX: 0, offsetY: 0 });
  const width = node.label.width();
  const height = node.label.height();
  const offsetX = textAnchorX(item.style.align, width);
  node.label.setAttrs({ offsetX, offsetY: height / 2 });
  const padding = textBoxPadding(item, output);
  node.box.setAttrs({
    x: node.label.x() - offsetX - padding,
    y: node.label.y() - height / 2 - padding,
    width: width + padding * 2,
    height: height + padding * 2,
    fill: item.style.decorationColor,
    visible: item.style.decoration === "box",
  });
}
