import Konva from "konva";

import { layerGeometry } from "../geometry";
import type { VideoOutput } from "../project";
import type { SceneLayer } from "./scene";
import type { LayerStyle } from "./scene-transition";

export interface LayerNode {
  group: Konva.Group;
  backdrop: Konva.Rect;
  picture: Konva.Image;
  curtain: Konva.Rect;
}

/**
 * One composed clip: the project background, the picture under its own transform, and the fade
 * curtain. Each layer carries its background so a slide or wipe moves a complete frame, exactly
 * like the two inputs of the export's `xfade` filter.
 */
export function createLayerNode(clipId: string): LayerNode {
  const group = new Konva.Group({ id: clipId });
  const backdrop = new Konva.Rect({ listening: false });
  const picture = new Konva.Image({
    image: undefined,
    name: "video-layer",
    id: `picture:${clipId}`,
    draggable: true,
  });
  const curtain = new Konva.Rect({ listening: false, fill: "#000000", opacity: 0 });
  group.add(backdrop, picture, curtain);
  return { group, backdrop, picture, curtain };
}

function frameAttrs(output: VideoOutput) {
  return { x: 0, y: 0, width: output.width, height: output.height };
}

function groupAttrs(style: LayerStyle) {
  return {
    x: style.dx,
    y: style.dy,
    opacity: style.opacity,
    clipX: style.clip?.x ?? 0,
    clipY: style.clip?.y ?? 0,
    clipWidth: style.clip?.width ?? 0,
    clipHeight: style.clip?.height ?? 0,
  };
}

function pictureAttrs(layer: SceneLayer, output: VideoOutput) {
  const geometry = layerGeometry(layer.asset, layer.transform, output);
  return {
    image: layer.image,
    x: geometry.centerX,
    y: geometry.centerY,
    offsetX: geometry.width / 2,
    offsetY: geometry.height / 2,
    width: geometry.width,
    height: geometry.height,
    rotation: layer.transform.rotation,
    scaleX: layer.transform.flipHorizontal ? -1 : 1,
    scaleY: layer.transform.flipVertical ? -1 : 1,
    opacity: layer.transform.opacity,
  };
}

export function updateLayerNode(
  node: LayerNode,
  layer: SceneLayer,
  frame: { output: VideoOutput; style: LayerStyle },
) {
  node.group.setAttrs(groupAttrs(frame.style));
  node.backdrop.setAttrs({ ...frameAttrs(frame.output), fill: frame.output.background });
  node.picture.setAttrs(pictureAttrs(layer, frame.output));
  node.curtain.setAttrs({ ...frameAttrs(frame.output), opacity: 1 - layer.fade });
}
