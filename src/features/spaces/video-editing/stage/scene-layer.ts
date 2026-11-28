import { Group } from "konva/lib/Group";
import { Rect } from "konva/lib/shapes/Rect";

import { layerGeometry } from "../geometry";
import type { VideoOutput } from "../project";
import type { SceneLayer } from "./scene";
import type { LayerStyle } from "./scene-transition";

export interface LayerNode {
  group: Group;
  picture: Rect;
}

/**
 * Transparent Konva hit proxy for a hardware-rendered clip layer. Konva handles pointer selection,
 * drag-to-reposition, resize, and rotation handles without redrawing video pixels on a 2D canvas.
 */
export function createLayerNode(clipId: string): LayerNode {
  const group = new Group({ id: clipId });
  const picture = new Rect({
    fill: "rgba(0,0,0,0)",
    name: "video-layer",
    id: `picture:${clipId}`,
    draggable: true,
  });
  group.add(picture);
  return { group, picture };
}

export function applyGroupStyle(group: Group, style: LayerStyle) {
  if (style.clip) {
    group.setAttrs({
      x: style.dx,
      y: style.dy,
      opacity: style.opacity,
      clipX: style.clip.x,
      clipY: style.clip.y,
      clipWidth: style.clip.width,
      clipHeight: style.clip.height,
    });
  } else {
    group.setAttrs({ x: style.dx, y: style.dy, opacity: style.opacity });
    if (typeof group.clipWidth() === "number") {
      group.clipWidth(null as unknown as number);
      group.clipHeight(null as unknown as number);
    }
  }
}

export function updateLayerNode(
  node: LayerNode,
  layer: SceneLayer,
  frame: { output: VideoOutput; style: LayerStyle },
) {
  applyGroupStyle(node.group, frame.style);
  const geometry = layerGeometry(layer.asset, layer.transform, frame.output);
  node.picture.setAttrs({
    x: geometry.centerX,
    y: geometry.centerY,
    offsetX: geometry.width / 2,
    offsetY: geometry.height / 2,
    width: geometry.width,
    height: geometry.height,
    rotation: layer.transform.rotation,
    scaleX: layer.transform.flipHorizontal ? -1 : 1,
    scaleY: layer.transform.flipVertical ? -1 : 1,
  });
}
