import type Konva from "konva";

import type { useStageViewport } from "../viewport/use-stage-viewport";
import type { createCanvasGestureSession } from "./canvas-gesture-session";
import { useCanvasEdgeAutoPan } from "./use-canvas-edge-auto-pan";

interface NodeDragAutoPanOptions {
  gestures: ReturnType<typeof createCanvasGestureSession>;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useNodeDragAutoPan(options: NodeDragAutoPanOptions) {
  let activeNodeId: string | undefined;
  const autoPan = useCanvasEdgeAutoPan({
    onPan: ({ canvasDelta }) => {
      if (!activeNodeId) return;
      options.gestures.controller.panDrag(canvasDelta);
    },
    setViewport: options.viewport.setViewport,
    size: options.viewport.size,
    viewport: options.viewport.viewport,
  });

  function begin(nodeId: string) {
    options.gestures.controller.beginDrag(nodeId);
    activeNodeId = nodeId;
    options.gestures.beginDirectManipulation("drag");
  }

  function move(nodeId: string, event: Konva.KonvaEventObject<DragEvent>) {
    if (activeNodeId !== nodeId) return;
    options.gestures.controller.moveDrag(nodeId);
    const point = event.target.getStage()?.getPointerPosition();
    if (point) autoPan.update(point);
  }

  async function end(nodeId: string) {
    autoPan.stop();
    try {
      await options.gestures.finish({ kind: "drag", nodeId });
    } finally {
      activeNodeId = undefined;
    }
  }

  function cancel() {
    autoPan.stop();
    activeNodeId = undefined;
  }

  return { begin, cancel, end, move };
}
