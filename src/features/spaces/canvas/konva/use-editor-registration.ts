import type Konva from "konva";
import { nextTick, onMounted, shallowRef, watch, type Ref } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { StructureCommand } from "../../organization/structure/commands/contract";
import type { CanvasNodeLayoutV1 } from "../contract/canvas-document";
import type { CanvasSurfaceEvents } from "../contract/canvas-editor";
import type { CanvasPoint, CanvasRect } from "../contract/canvas-editor-port";
import type { CanvasFrameMutation } from "./canvas-editor-port-options";
import { createKonvaEditorPort } from "./create-canvas-editor-port";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { CanvasPalette } from "./presentation/use-canvas-palette";
import type { useCanvasSelection } from "./selection/use-canvas-selection";
import type { useStageViewport } from "./viewport/use-stage-viewport";

interface EditorRegistrationOptions {
  cancelInteraction(): void;
  executeStructureCommand(command: StructureCommand): Promise<void>;
  enterGroup(id: string): void;
  exitGroup(): void;
  scopeGroupId(): string | null;
  setEditingScope(ids: readonly string[]): void;
  requestContainerRename(id: string): void;
  events: CanvasSurfaceEvents;
  focusNodeIds(): readonly string[];
  host: Readonly<Ref<HTMLElement | null>>;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
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
  selection: ReturnType<typeof useCanvasSelection>;
  stage: Ref<Konva.Stage | undefined>;
  stageComponent: Readonly<Ref<VueKonvaRef<Konva.Stage> | null>>;
  viewport: ReturnType<typeof useStageViewport>;
}

export function useEditorRegistration(options: EditorRegistrationOptions) {
  const editorRef = shallowRef<ReturnType<typeof createKonvaEditorPort>>();
  let editor: ReturnType<typeof createKonvaEditorPort> | undefined;
  watch(options.focusNodeIds, (ids) => editor?.reveal(ids, "focus"));
  onMounted(async () => {
    await nextTick();
    options.stage.value = options.stageComponent.value?.getStage();
    if (!options.host.value) throw new Error("Konva canvas host is not mounted");
    editor = createKonvaEditorPort({
      ...options.events,
      cancelInteraction: options.cancelInteraction,
      executeStructureCommand: options.executeStructureCommand,
      enterGroup: options.enterGroup,
      exitGroup: options.exitGroup,
      scopeGroupId: options.scopeGroupId,
      setEditingScope: options.setEditingScope,
      requestContainerRename: options.requestContainerRename,
      runFrameMutation: options.runFrameMutation,
      host: () => options.host.value!,
      layouts: () => options.layouts.value,
      model: options.model,
      nodeById: options.nodeById,
      nudgeSelection: options.nudgeSelection,
      nodes: options.nodes,
      palette: options.palette,
      pastePoint: options.pastePoint,
      relationIds: () => options.selection.relationIds.value,
      relationBounds: options.relationBounds,
      relations: options.relations,
      selectableNodeIds: options.selectableNodeIds,
      selectedNodeIds: () => options.selection.nodeIds.value,
      setSelection: options.selection.setNodeIds,
      animateViewport: options.viewport.animateViewport,
      setViewport: options.viewport.setViewport,
      size: () => options.viewport.size,
      viewport: options.viewport.getViewport,
    });
    editorRef.value = editor;
    options.events.onReady(editor);
    editor.reveal(options.focusNodeIds(), "focus");
  });
  return editorRef;
}
