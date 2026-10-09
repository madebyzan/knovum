import type Konva from "konva";
import type { Ref } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasNodeLayoutV1 } from "../schema/document";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { createCanvasGestureSession } from "./interaction/canvas-gesture-session";
import type { usePointerInteractions } from "./interaction/use-pointer-interactions";
import type { CanvasPalette } from "./presentation/use-canvas-palette";
import type { CanvasScene } from "./scene/canvas-scene";
import type { useCanvasSelection } from "./selection/use-canvas-selection";
import type { useCanvasSceneProjections } from "./use-canvas-scene-projections";
import { useEditorRegistration } from "./use-editor-registration";
import { canvasPastePoint } from "./viewport/canvas-paste-point";
import type { useStageViewport } from "./viewport/use-stage-viewport";

interface RuntimeEditorRegistrationOptions {
  cancelInteraction(): void;
  gestures: ReturnType<typeof createCanvasGestureSession>;
  host: Readonly<Ref<HTMLElement | null>>;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  model: CanvasDocumentModel;
  palette: Readonly<Ref<CanvasPalette>>;
  pointer: ReturnType<typeof usePointerInteractions>;
  projections: ReturnType<typeof useCanvasSceneProjections>;
  props: CanvasSurfaceProps;
  scene: Readonly<Ref<CanvasScene>>;
  selection: ReturnType<typeof useCanvasSelection>;
  stage: Ref<Konva.Stage | undefined>;
  stageComponent: Readonly<Ref<VueKonvaRef<Konva.Stage> | null>>;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useRuntimeEditorRegistration(options: RuntimeEditorRegistrationOptions) {
  const { props } = options;
  useEditorRegistration({
    cancelInteraction: options.cancelInteraction,
    events: props.events,
    focusNodeIds: () => props.focusNodeIds,
    host: options.host,
    layouts: options.layouts,
    lockedNodeIds: () => options.projections.lockedNodeIds.value,
    model: options.model,
    nodeById: (nodeId) => options.scene.value.nodesById.get(nodeId)?.node,
    runFrameMutation: options.gestures.runFrameMutation,
    nudgeSelection: options.gestures.nudge,
    nodes: () => props.sceneInput.nodes,
    palette: () => options.palette.value,
    relations: () => props.sceneInput.relations,
    scene: () => options.scene.value,
    relationBoundsById: () => options.projections.relationBoundsById.value,
    pastePoint: () =>
      canvasPastePoint(
        options.pointer.lastCanvasPoint.value,
        options.viewport.viewport.value,
        options.viewport.size,
      ),
    selection: options.selection,
    snapshot: () => props.sceneInput.editSnapshot,
    structureBoundsById: () => options.projections.structureBoundsById.value,
    selectableNodeIds: () => options.scene.value.nodes.map(({ node }) => node.id),
    stage: options.stage,
    stageComponent: options.stageComponent,
    viewport: options.viewport,
  });
}
