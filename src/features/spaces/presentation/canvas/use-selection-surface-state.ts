import { computed, shallowRef, watch } from "vue";

import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useImageRemovalSession } from "../../knowledge-derivations/image-removal-session";
import { useSpaceStore } from "../../session/space-store";

/** Keeps selection controls attached to the owning image interaction. */
export function useSelectionSurfaceState() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const nodeOperations = useNodeOperationGate();
  const imageEdit = useImageEditSession();
  const imageRemoval = useImageRemovalSession();
  const angleNodeId = shallowRef<string>();

  const selectionKey = computed(() => {
    const { nodeIds, structureNodeIds, relationIds } = editorSession.selection.value;
    return [...nodeIds, ...structureNodeIds, ...relationIds].join("|");
  });
  const kind = computed(() => {
    const tool = imageEdit.active?.tool;
    if (tool === "adjust" || tool === "crop" || angleNodeId.value) return "inspector";
    if (imageRemoval.active || nodeOperations.active.value?.kind === "image-comment") return "compact-editor";
    return tool ? "compact-editor" : "actions";
  });
  const showActions = computed(() => {
    if (store.workflowMode) return false;
    const operation = nodeOperations.active.value;
    return (
      !operation ||
      operation.kind === "image-edit" ||
      operation.kind === "image-removal" ||
      operation.kind === "image-comment"
    );
  });
  watch(
    [selectionKey, () => imageEdit.active?.tool, () => imageRemoval.active?.nodeId, () => store.workflowMode],
    () => (angleNodeId.value = undefined),
  );
  return { angleNodeId, selectionKey, kind, showActions };
}
