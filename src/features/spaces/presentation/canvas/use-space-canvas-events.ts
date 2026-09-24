import type { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasHostEvents } from "../../canvas/contract/canvas-editor";
import type { CanvasEditorPort } from "../../canvas/contract/canvas-editor-port";
import { updateCanvasSelectionProjection, updateCanvasViewportProjection } from "../../canvas/editor-session";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { createStructureIndex } from "../../model/structure/structure-index";
import { groupScopePathFor } from "../../model/structure/structure-queries";
import { sectionDrawCandidateIds } from "../../organization/section-draw-candidates";
import type { createCanvasSelectionActions } from "../../session/actions/canvas-selection-actions";
import type { useSpaceStore } from "../../session/space-store";
import type { CanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import type { useSpaceCanvasImports } from "./use-space-canvas-imports";

export function createSpaceCanvasEvents(options: {
  activateNode(nodeId: string): Promise<void>;
  attachEditor(editor: CanvasEditorPort): void;
  contentLayout: CanvasContentLayoutCoordinator;
  createTextNode: CanvasHostEvents["onCreateTextNode"];
  imports: ReturnType<typeof useSpaceCanvasImports>;
  reportError(error: unknown): void;
  selection: ReturnType<typeof createCanvasSelectionActions>;
  store: ReturnType<typeof useSpaceStore>;
  tools: ReturnType<typeof useCanvasToolStore>;
  untitledSection: string;
}): CanvasHostEvents {
  const { store } = options;
  const operations = useNodeOperationGate();
  return {
    onCreateTextNode: (kind, point) =>
      operations.allowsGestureCommit() ? options.createTextNode(kind, point) : Promise.resolve(undefined),
    async onCreateSection(frame) {
      if (!operations.allowsGestureCommit()) return;
      await store.executeSpaceEdit({
        kind: "section.draw",
        sectionId: crypto.randomUUID(),
        title: options.untitledSection,
        frame,
        containedRootIds: sectionDrawCandidateIds(store.editSnapshot, frame),
      });
      options.tools.resetTool();
    },
    onExecuteSpaceEdit: (command) =>
      operations.allowsGestureCommit() ? store.executeSpaceEdit(command) : Promise.resolve(),
    onEnterGroup(structureNodeId) {
      store.groupScopePath = groupScopePathFor(
        structureNodeId,
        createStructureIndex(store.editSnapshot.structure),
      ).concat(structureNodeId);
    },
    onExitGroup() {
      if (!store.groupScopePath.length) return false;
      store.groupScopePath = store.groupScopePath.slice(0, -1);
      return true;
    },
    onCopyNodes: options.selection.copyNodes,
    onDeleteRelations: (ids) =>
      operations.allowsGestureCommit() ? store.removeRelations(ids) : Promise.resolve(),
    onCreateRelation: async (sourceNodeId, targetNodeId) => {
      if (!operations.allowsGestureCommit()) return;
      await store.createRelation(sourceNodeId, targetNodeId, "related");
    },
    onInsertText: options.imports.insertText,
    onInsertUrl: options.imports.insertUrl,
    onImportFiles: options.imports.importFiles,
    onNodeActivate: options.activateNode,
    onPasteNodes: options.imports.pasteNodes,
    onError: options.reportError,
    onHistoryTransaction: store.recordHistoryMutation,
    onDirectManipulationChange: options.contentLayout.updateDirectManipulation,
    onMutationPendingChange: options.contentLayout.updateMutationPending,
    onReady: options.attachEditor,
    onSelectionChange: updateCanvasSelectionProjection,
    onViewportChange: updateCanvasViewportProjection,
  };
}
