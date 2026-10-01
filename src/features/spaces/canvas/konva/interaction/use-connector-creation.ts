import type Konva from "konva";
import { computed, shallowRef, type Ref } from "vue";

import type { CanvasHostEvents } from "../../contracts/canvas-editor";
import type { CanvasScene } from "../scene/canvas-scene";
import type { useStageViewport } from "../viewport/use-stage-viewport";
import { canvasPointFromScreen } from "../viewport/viewport-math";
import { ConnectorGestureController, type CanvasConnectorDraft } from "./connector-gesture-controller";
import { createConnectorTargetIndex } from "./connector-target-index";
import { useCanvasEdgeAutoPan } from "./use-canvas-edge-auto-pan";
import type { usePointerInteractions } from "./use-pointer-interactions";

interface ConnectorCreationOptions {
  createRelation: CanvasHostEvents["onCreateRelation"];
  reportError(error: unknown): void;
  pointer: ReturnType<typeof usePointerInteractions>;
  scene: Readonly<Ref<CanvasScene>>;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useConnectorCreation(options: ConnectorCreationOptions) {
  const draft = shallowRef<CanvasConnectorDraft>();
  const targetIndex = computed(() =>
    createConnectorTargetIndex(
      options.scene.value.nodes
        .filter(({ interaction }) => interaction.connectable)
        .map(({ layout }) => layout),
    ),
  );
  const controller = new ConnectorGestureController(
    () => targetIndex.value,
    (value) => (draft.value = value),
    () => options.viewport.viewport.value.zoom,
  );
  const autoPan = useCanvasEdgeAutoPan({
    onPan: ({ screenPoint }) =>
      controller.move(canvasPointFromScreen(screenPoint, options.viewport.viewport.value)),
    setViewport: options.viewport.setViewport,
    size: options.viewport.size,
    viewport: options.viewport.viewport,
  });

  function pointerMove(event: Konva.KonvaEventObject<PointerEvent>) {
    const stage = event.target.getStage();
    const point = stage?.getRelativePointerPosition();
    const screenPoint = stage?.getPointerPosition();
    if (!point || !controller.move(point)) options.pointer.pointerMove(event);
    else if (screenPoint) autoPan.update(screenPoint);
  }

  function pointerUp(event: Konva.KonvaEventObject<PointerEvent>) {
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (point && controller.active) {
      finish(point);
      return;
    }
    options.pointer.pointerUp(event.evt);
  }

  function cancel() {
    autoPan.stop();
    controller.cancel();
  }

  function finish(point: { x: number; y: number }) {
    autoPan.stop();
    const connector = controller.finish(point);
    if (connector) {
      void options.createRelation(connector.sourceNodeId, connector.targetNodeId).catch(options.reportError);
    }
  }

  return {
    begin: (nodeId: string, side: Parameters<ConnectorGestureController["begin"]>[1]) =>
      controller.begin(nodeId, side),
    cancel,
    draft,
    pointerMove,
    pointerUp,
  };
}
