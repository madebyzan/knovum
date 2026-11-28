import type { Group } from "konva/lib/Group";
import { Layer } from "konva/lib/Layer";
import { Stage } from "konva/lib/Stage";

import type { VideoOutput } from "../project";
import { createDomVideoLayer } from "./dom-video-layer";
import type { Scene } from "./scene";
import { createLayerNode, updateLayerNode, type LayerNode } from "./scene-layer";
import { createTextNode, updateTextNode, type TextNode } from "./scene-text-node";
import { PLAIN_LAYER, transitionStyle, type TransitionStyle } from "./scene-transition";

function prune(nodeById: Map<string, { group: Group }>, keep: Set<string>) {
  for (const [id, node] of nodeById) {
    if (keep.has(id)) continue;
    node.group.destroy();
    nodeById.delete(id);
  }
}

function styleOf(transition: TransitionStyle | undefined, index: number) {
  if (!transition) return PLAIN_LAYER;
  return index === 0 ? transition.from : transition.to;
}

function sameKonvaLayer(a: Scene["layers"][number], b: Scene["layers"][number]): boolean {
  return a.id === b.id && a.transform === b.transform && a.asset === b.asset;
}

function sameTransition(a: Scene["transition"], b: Scene["transition"]): boolean {
  return a?.kind === b?.kind && a?.progress === b?.progress;
}

function sameKonvaScene(previous: Scene | undefined, next: Scene): boolean {
  if (!previous || previous.output !== next.output) return false;
  if (previous.layers.length !== next.layers.length || previous.texts.length !== next.texts.length) {
    return false;
  }
  if (!sameTransition(previous.transition, next.transition)) return false;
  return (
    next.layers.every((layer, index) => sameKonvaLayer(previous.layers[index]!, layer)) &&
    next.texts.every((text, index) => previous.texts[index] === text)
  );
}

/**
 * Hybrid hardware stage scene: native GPU `<video>` and still layers live in `domVideoLayer`
 * underneath a transparent Konva Stage that owns caption rendering, hit-testing, and Transformer
 * handles. During playback, Konva performs zero 2D redraws between caption or layout changes.
 */
export function createStageScene(container: HTMLDivElement) {
  const domLayer = createDomVideoLayer(container);
  const konvaHost = document.createElement("div");
  konvaHost.style.cssText = "position:absolute;inset:0;";
  container.appendChild(konvaHost);

  const stage = new Stage({ container: konvaHost, width: 2, height: 2 });
  const content = new Layer();
  const overlay = new Layer();
  const layerById = new Map<string, LayerNode>();
  const textById = new Map<string, TextNode>();
  let applied = "";
  let currentScale = 1;
  let lastKonvaScene: Scene | undefined;
  let lastDomScene: Scene | undefined;
  stage.add(content);
  stage.add(overlay);

  function syncLayers(scene: Scene, transition: TransitionStyle | undefined) {
    const order: Group[] = [];
    for (const [index, layer] of scene.layers.entries()) {
      let node = layerById.get(layer.id);
      if (!node) {
        node = createLayerNode(layer.id);
        layerById.set(layer.id, node);
        content.add(node.group);
      }
      updateLayerNode(node, layer, { output: scene.output, style: styleOf(transition, index) });
      order.push(node.group);
    }
    prune(layerById, new Set(scene.layers.map(({ id }) => id)));
    return order;
  }

  function syncTexts(scene: Scene) {
    const order: Group[] = [];
    for (const item of scene.texts) {
      let node = textById.get(item.id);
      if (!node) {
        node = createTextNode(item.id);
        textById.set(item.id, node);
        content.add(node.group);
      }
      updateTextNode(node, item, scene.output);
      order.push(node.group);
    }
    prune(textById, new Set(scene.texts.map(({ id }) => id)));
    return order;
  }

  return {
    stage,
    content,
    overlay,
    present(scene: Scene) {
      lastDomScene = scene;
      domLayer.present(scene, currentScale);
      if (sameKonvaScene(lastKonvaScene, scene)) return;
      lastKonvaScene = scene;
      const transition = scene.transition ? transitionStyle(scene.transition, scene.output) : undefined;
      const ordered = [...syncLayers(scene, transition), ...syncTexts(scene)];
      for (const [index, node] of ordered.entries()) node.zIndex(index);
      content.batchDraw();
    },
    /** Sizes the surface to the pixels it occupies while every coordinate stays in output space. */
    resize(output: VideoOutput, scale: number) {
      const identity = `${output.width}x${output.height}@${scale}`;
      if (identity === applied) return;
      applied = identity;
      currentScale = scale;
      lastKonvaScene = undefined;
      stage.size({
        width: Math.max(2, Math.round(output.width * scale)),
        height: Math.max(2, Math.round(output.height * scale)),
      });
      stage.scale({ x: scale, y: scale });
      if (lastDomScene) domLayer.present(lastDomScene, currentScale);
    },
    dispose() {
      lastKonvaScene = undefined;
      lastDomScene = undefined;
      layerById.clear();
      textById.clear();
      stage.destroy();
      konvaHost.remove();
      domLayer.dispose();
    },
  };
}

export type StageScene = ReturnType<typeof createStageScene>;
