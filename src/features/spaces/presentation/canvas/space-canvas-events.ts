import { i18n } from "../../../../foundation/i18n/instance";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasHostEvents } from "../../canvas/contracts/canvas-editor";
import type { CanvasEditorPort } from "../../canvas/contracts/canvas-editor-port";
import { updateCanvasSelectionProjection, updateCanvasViewportProjection } from "../../canvas/editor-session";
import type { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { createStructureIndex } from "../../model/structure/structure-index";
import { groupScopePathFor } from "../../model/structure/structure-queries";
import type { createCanvasSelectionActions } from "../../session/actions/canvas-selection-actions";
import { useSpaceStore } from "../../session/space-store";
import { sectionDrawCandidateIds } from "../../structure/section-draw-candidates";
import type { CanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import type { useSpaceCanvasImports } from "./use-space-canvas-imports";

interface SpaceCanvasEventOptions {
  activateNode(nodeId: string): Promise<void>;
  attachEditor(editor: CanvasEditorPort): void;
  contentLayout: CanvasContentLayoutCoordinator;
  createTextNode: CanvasHostEvents["onCreateTextNode"];
  imports: ReturnType<typeof useSpaceCanvasImports>;
  operations: ReturnType<typeof useNodeOperationGate>;
  reportError(error: unknown): void;
  selection: ReturnType<typeof createCanvasSelectionActions>;
}

export function createSpaceCanvasEvents(options: SpaceCanvasEventOptions): CanvasHostEvents {
  const store = useSpaceStore();
  const tools = useCanvasToolStore();

  const { operations } = options;
  return {
    onCreateTextNode: (kind, point) =>
      operations.allowsGestureCommit() ? options.createTextNode(kind, point) : Promise.resolve(undefined),
    async onCreateSection(frame) {
      if (!operations.allowsGestureCommit()) return;
      await store.executeSpaceEdit({
        kind: "section.draw",
        sectionId: crypto.randomUUID(),
        title: i18n.global.t("spaces.canvas.sections.untitled"),
        frame,
        containedRootIds: sectionDrawCandidateIds(store.editSnapshot, frame),
      });
      tools.resetTool();
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
    onDeleteRelations: (relationIds) =>
      operations.allowsGestureCommit() ? store.revokeRelations(relationIds) : Promise.resolve(),
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
