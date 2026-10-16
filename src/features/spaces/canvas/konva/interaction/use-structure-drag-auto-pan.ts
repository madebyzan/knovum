import type Konva from "konva";

import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../../../model/structure/structure-index";
import type { useStageViewport } from "../viewport/use-stage-viewport";
import { useCanvasEdgeAutoPan } from "./use-canvas-edge-auto-pan";
import type { useStructureGestureSession } from "./use-structure-gesture-session";

interface StructureDragAutoPanOptions {
  gestures: ReturnType<typeof useStructureGestureSession>;
  snapshot(): SpaceEditSnapshot;
  onBegin(): void;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useStructureDragAutoPan(options: StructureDragAutoPanOptions) {
  let active = false;
  const autoPan = useCanvasEdgeAutoPan({
    onPan: () => {
      if (!active) return;
      options.gestures.move();
    },
    setViewport: options.viewport.setViewport,
    size: options.viewport.size,
    viewport: options.viewport.getViewport,
  });

  function begin(structureNodeId: string, event: Konva.KonvaEventObject<DragEvent>) {
    active = true;
    options.onBegin();
    options.gestures.beginMove(structureNodeId, event.target);
  }

  function beginNode(nodeId: string, event: Konva.KonvaEventObject<DragEvent>) {
    const id = createStructureIndex(options.snapshot().structure).structureNodeIdByNodeId.get(nodeId);
    if (!id) throw new Error(`Content structure is unavailable: ${nodeId}`);
    begin(id, event);
  }

  function move(event: Konva.KonvaEventObject<DragEvent>) {
    if (!active) return;
    options.gestures.move();
    const point = event.target.getStage()?.getPointerPosition();
    if (point) autoPan.update(point);
  }

  async function end() {
    autoPan.stop();
    active = false;
    await options.gestures.finish();
  }

  function cancel() {
    autoPan.stop();
    active = false;
  }

  return { begin, beginNode, cancel, end, move };
}
