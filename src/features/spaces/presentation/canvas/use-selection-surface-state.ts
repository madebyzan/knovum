import { computed, shallowRef, watch } from "vue";

import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { useImageEditSessionStore } from "../../image-editing/image-edit-session-store";
import { useImageRemovalSessionStore } from "../../knowledge-derivations/image-removal-session-store";
import { useSpaceStore } from "../../session/space-store";
import { NODE_SPEECH_OPERATIONS, type NodeSpeechOperation } from "../speech/node-speech-operations";

/** Keeps selection controls attached to the owning capability interaction. */
export function useSelectionSurfaceState() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const nodeOperations = useNodeOperationGate();
  const imageEdit = useImageEditSessionStore();
  const imageRemoval = useImageRemovalSessionStore();
  const angleNodeId = shallowRef<string>();
  const speechOperation = shallowRef<NodeSpeechOperation>();

  const selectionKey = computed(() => {
    const { nodeIds, structureNodeIds, relationIds } = editorSession.selection.value;
    return [...nodeIds, ...structureNodeIds, ...relationIds].join("|");
  });
  const kind = computed(() => {
    const tool = imageEdit.active?.tool;
    if (speechOperation.value) return NODE_SPEECH_OPERATIONS[speechOperation.value].surface;
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
  watch([selectionKey, () => store.activeSpaceId], () => (speechOperation.value = undefined));
  return { angleNodeId, speechOperation, selectionKey, kind, showActions };
}
