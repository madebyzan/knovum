import type Konva from "konva";
import { nextTick, onMounted, watch, type Ref } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasHostEvents } from "../contracts/canvas-editor";
import type { CanvasPoint, CanvasRect } from "../contracts/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../schema/document";
import type { CanvasFrameMutation } from "./canvas-editor-port-options";
import { createKonvaEditorPort } from "./create-canvas-editor-port";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { CanvasPalette } from "./presentation/use-canvas-palette";
import type { CanvasScene } from "./scene/canvas-scene";
import type { useCanvasSelection } from "./selection/use-canvas-selection";
import type { useStageViewport } from "./viewport/use-stage-viewport";

interface EditorRegistrationOptions {
  cancelInteraction(): void;
  events: CanvasHostEvents;
  focusNodeIds(): readonly string[];
  host: Readonly<Ref<HTMLElement | null>>;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  lockedNodeIds(): ReadonlySet<string>;
  model: CanvasDocumentModel;
  nodeById(id: string): SpaceCanvasNode | undefined;
  runFrameMutation(mutation: CanvasFrameMutation): Promise<void>;
  nudgeSelection(offset: CanvasPoint): Promise<void>;
  nodes(): readonly SpaceCanvasNode[];
  palette(): CanvasPalette;
  relations(): readonly SpaceRelation[];
  relationBounds(): ReadonlyMap<string, CanvasRect>;
  pastePoint(): CanvasPoint;
  selectableNodeIds(): readonly string[];
  scene(): CanvasScene;
  snapshot(): SpaceEditSnapshot;
  structureBounds(): ReadonlyMap<string, CanvasRect>;
  selection: ReturnType<typeof useCanvasSelection>;
  stage: Ref<Konva.Stage | undefined>;
  stageComponent: Readonly<Ref<VueKonvaRef<Konva.Stage> | null>>;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useEditorRegistration(options: EditorRegistrationOptions) {
  let editor: ReturnType<typeof createKonvaEditorPort> | undefined;
  watch(options.focusNodeIds, (ids) => editor?.reveal(ids, "focus"));
  onMounted(async () => {
    await nextTick();
    options.stage.value = options.stageComponent.value?.getStage();
    if (!options.host.value) throw new Error("Konva canvas host is not mounted");
    editor = createKonvaEditorPort({
      ...options.events,
      cancelInteraction: options.cancelInteraction,
      runFrameMutation: options.runFrameMutation,
      host: () => options.host.value!,
      layouts: () => options.layouts.value,
      lockedNodeIds: options.lockedNodeIds,
      model: options.model,
      nodeById: options.nodeById,
      nudgeSelection: options.nudgeSelection,
      nodes: options.nodes,
      palette: options.palette,
      pastePoint: options.pastePoint,
      relationIds: () => options.selection.relationIds.value,
      relationBounds: options.relationBounds,
      relations: options.relations,
      scene: options.scene,
      selectableNodeIds: options.selectableNodeIds,
      selectedNodeIds: () => options.selection.nodeIds.value,
      selectedStructureNodeIds: () => options.selection.structureNodeIds.value,
      setSelection: options.selection.setNodeIds,
      setStructureSelection: options.selection.setStructureNodeIds,
      snapshot: options.snapshot,
      structureBounds: options.structureBounds,
      animateViewport: options.viewport.animateViewport,
      setViewport: options.viewport.setViewport,
      size: () => options.viewport.size,
      viewport: options.viewport.getViewport,
    });
    options.events.onReady(editor);
    editor.reveal(options.focusNodeIds(), "focus");
  });
}
