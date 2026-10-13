import Konva from "konva";

import type { VideoOutput } from "../project";
import type { Scene } from "./scene";
import { createLayerNode, updateLayerNode, type LayerNode } from "./scene-layer";
import { createTextNode, updateTextNode, type TextNode } from "./scene-text-node";
import { PLAIN_LAYER, transitionStyle, type TransitionStyle } from "./scene-transition";

function prune(nodes: Map<string, { group: Konva.Group }>, keep: Set<string>) {
  for (const [id, node] of nodes) {
    if (keep.has(id)) continue;
    node.group.destroy();
    nodes.delete(id);
  }
}

function styleOf(transition: TransitionStyle | undefined, index: number) {
  if (!transition) return PLAIN_LAYER;
  return index === 0 ? transition.from : transition.to;
}

/**
 * The preview scene graph. Konva owns the nodes, their transforms, hit areas and redraw batching;
 * this module only translates a scene description into nodes and keeps one node per clip and
 * overlay so a frame update never rebuilds the graph.
 */
export function createStageScene(container: HTMLDivElement) {
  const stage = new Konva.Stage({ container, width: 2, height: 2 });
  const content = new Konva.Layer();
  const overlay = new Konva.Layer();
  const backdrop = new Konva.Rect({ listening: false });
  const layers = new Map<string, LayerNode>();
  const texts = new Map<string, TextNode>();
  let applied = "";
  content.add(backdrop);
  stage.add(content);
  stage.add(overlay);

  function syncLayers(scene: Scene, transition: TransitionStyle | undefined) {
    const order: Konva.Group[] = [];
    for (const [index, layer] of scene.layers.entries()) {
      let node = layers.get(layer.id);
      if (!node) {
        node = createLayerNode(layer.id);
        layers.set(layer.id, node);
        content.add(node.group);
      }
      updateLayerNode(node, layer, { output: scene.output, style: styleOf(transition, index) });
      order.push(node.group);
    }
    prune(layers, new Set(scene.layers.map(({ id }) => id)));
    return order;
  }

  function syncTexts(scene: Scene) {
    const order: Konva.Group[] = [];
    for (const item of scene.texts) {
      let node = texts.get(item.id);
      if (!node) {
        node = createTextNode(item.id);
        texts.set(item.id, node);
        content.add(node.group);
      }
      updateTextNode(node, item, scene.output);
      order.push(node.group);
    }
    prune(texts, new Set(scene.texts.map(({ id }) => id)));
    return order;
  }

  return {
    stage,
    content,
    overlay,
    present(scene: Scene) {
      const transition = scene.transition ? transitionStyle(scene.transition, scene.output) : undefined;
      backdrop.setAttrs({
        x: 0,
        y: 0,
        width: scene.output.width,
        height: scene.output.height,
        fill: transition?.backdrop ?? scene.output.background,
      });
      const ordered = [backdrop, ...syncLayers(scene, transition), ...syncTexts(scene)];
      for (const [index, node] of ordered.entries()) node.zIndex(index);
      content.batchDraw();
    },
    /** Sizes the surface to the pixels it occupies while every coordinate stays in output space. */
    resize(output: VideoOutput, scale: number) {
      const identity = `${output.width}x${output.height}@${scale}`;
      if (identity === applied) return;
      applied = identity;
      stage.size({
        width: Math.max(2, Math.round(output.width * scale)),
        height: Math.max(2, Math.round(output.height * scale)),
      });
      stage.scale({ x: scale, y: scale });
    },
    dispose() {
      layers.clear();
      texts.clear();
      stage.destroy();
    },
  };
}

export type StageScene = ReturnType<typeof createStageScene>;
