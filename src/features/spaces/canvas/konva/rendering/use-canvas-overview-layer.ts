import { warn as logWarn } from "@tauri-apps/plugin-log";
import Konva from "konva";
import { onScopeDispose, watch, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { canvasOverviewImageSource, loadCanvasOverviewImage } from "../overview/assets";
import { canvasOverviewLabelled, createCanvasOverviewNode } from "../overview/nodes";
import type { CanvasSceneNode } from "../scene/canvas-scene";
import type { CanvasPalette } from "./use-canvas-palette";

interface CanvasOverviewLayerOptions {
  layer: Readonly<Ref<Konva.Layer | undefined>>;
  /** The visible nodes that do not show their content, in paint order. */
  nodes: Readonly<Ref<readonly CanvasSceneNode[]>>;
  layoutPreviewByNodeId: () => ReadonlyMap<string, CanvasNodeLayoutV1>;
  palette: Readonly<Ref<CanvasPalette>>;
  zoom: Readonly<Ref<number>>;
}

interface OverviewDrawing {
  sceneNode: CanvasSceneNode;
  palette: CanvasPalette;
  labelled: boolean;
  source: string | undefined;
}

interface DrawnOverview extends OverviewDrawing {
  group: Konva.Group;
  image: ImageBitmap | undefined;
}

/**
 * Draws the visible nodes that do not mount their content as their overview, on one Konva layer.
 *
 * A node is redrawn only when what its drawing depends on changes — its content, size, appearance,
 * palette, whether it is large enough to be labelled, or its image arriving — and otherwise only
 * follows its frame, so zooming and panning cost a pass over the drawn nodes, not a rebuild of them.
 */
export function useCanvasOverviewLayer(options: CanvasOverviewLayerOptions) {
  const drawnByNodeId = new Map<string, DrawnOverview>();
  let order: readonly string[] = [];

  function draw(drawn: DrawnOverview) {
    drawn.group.destroyChildren();
    drawn.group.add(
      createCanvasOverviewNode({
        sceneNode: drawn.sceneNode,
        palette: drawn.palette,
        labelled: drawn.labelled,
        ...(drawn.image ? { image: drawn.image } : {}),
      }),
    );
  }

  function requestImage(nodeId: string, source: string) {
    void loadCanvasOverviewImage(source)
      .then((image) => {
        const drawn = drawnByNodeId.get(nodeId);
        if (drawn?.source !== source || drawn.image === image) return undefined;
        drawn.image = image;
        draw(drawn);
        drawn.group.getLayer()?.batchDraw();
        return undefined;
      })
      // A preview that cannot be decoded leaves the node drawn without it, as a node that has none.
      .catch(
        (error: unknown) => void logWarn(`Canvas overview could not load a preview image: ${String(error)}`),
      );
  }

  function drawingOf(sceneNode: CanvasSceneNode, previews: ReadonlyMap<string, CanvasNodeLayoutV1>) {
    const layout = previews.get(sceneNode.node.id) ?? sceneNode.layout;
    const drawn = layout === sceneNode.layout ? sceneNode : { ...sceneNode, layout };
    return {
      sceneNode: drawn,
      palette: options.palette.value,
      labelled: canvasOverviewLabelled(drawn, options.zoom.value),
      source: canvasOverviewImageSource(sceneNode),
    } satisfies OverviewDrawing;
  }

  /** Brings a node's drawn overview up to `drawing`, drawing it only when what it shows changed. */
  function refresh(nodeId: string, drawing: OverviewDrawing) {
    const drawn = drawnByNodeId.get(nodeId);
    if (drawn && sameDrawing(drawn, drawing)) {
      drawn.sceneNode = drawing.sceneNode;
      return drawn;
    }
    const next: DrawnOverview = drawn
      ? Object.assign(drawn, drawing, { image: drawn.source === drawing.source ? drawn.image : undefined })
      : { ...drawing, group: new Konva.Group({ listening: false }), image: undefined };
    drawnByNodeId.set(nodeId, next);
    draw(next);
    if (next.source && !next.image) requestImage(nodeId, next.source);
    return next;
  }

  function reorder(layer: Konva.Layer, nextOrder: readonly string[]) {
    if (nextOrder.length === order.length && nextOrder.every((id, index) => id === order[index])) return;
    layer.removeChildren();
    for (const id of nextOrder) layer.add(drawnByNodeId.get(id)!.group);
    order = nextOrder;
  }

  function update(layer: Konva.Layer) {
    const previews = options.layoutPreviewByNodeId();
    const drawnIds = new Set<string>();
    for (const sceneNode of options.nodes.value) {
      const drawing = drawingOf(sceneNode, previews);
      const { frame } = drawing.sceneNode.layout;
      refresh(sceneNode.node.id, drawing).group.setAttrs({
        x: frame.x,
        y: frame.y,
        rotation: frame.rotation,
      });
      drawnIds.add(sceneNode.node.id);
    }
    for (const [nodeId, drawn] of drawnByNodeId) {
      if (drawnIds.has(nodeId)) continue;
      drawn.group.destroy();
      drawnByNodeId.delete(nodeId);
    }
    reorder(
      layer,
      options.nodes.value.map(({ node }) => node.id),
    );
    layer.batchDraw();
  }

  watch(
    [options.layer, options.nodes, options.layoutPreviewByNodeId, options.palette, options.zoom],
    ([layer]) => {
      if (layer) update(layer);
    },
    { flush: "post", immediate: true },
  );
  onScopeDispose(() => {
    for (const drawn of drawnByNodeId.values()) drawn.group.destroy();
    drawnByNodeId.clear();
  });
}

function sameDrawing(current: OverviewDrawing, next: OverviewDrawing) {
  const before = current.sceneNode.layout;
  const after = next.sceneNode.layout;
  return (
    current.sceneNode.node === next.sceneNode.node &&
    current.palette === next.palette &&
    current.labelled === next.labelled &&
    current.source === next.source &&
    before.frame.width === after.frame.width &&
    before.frame.height === after.frame.height &&
    before.appearance === after.appearance
  );
}
