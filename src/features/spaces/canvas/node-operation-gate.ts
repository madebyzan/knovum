import { computed } from "vue";

import { useImageCommentSession } from "../comments/image-comment-session";
import { useImageEditSession } from "../image-editing/image-edit-session";
import { useImageRemovalSession } from "../knowledge-derivations/image-removal-session";
import { useNodeOperationDrafts } from "../session/node-operation-drafts";
import { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

export type NodeOperationEffect = "inspect" | "edit" | "structure" | "create" | "clone" | "derive" | "export";
export type ActiveNodeOperation =
  | { kind: "image-edit"; nodeId: string }
  | { kind: "image-removal"; nodeId: string }
  | { kind: "image-comment"; nodeId: string }
  | { kind: "text-edit"; nodeId: string }
  | { kind: "document-edit"; nodeId: string }
  | { kind: "video-edit"; nodeId: string }
  | { kind: "canvas-gesture" }
  | { kind: "node-draft"; nodeId: string };

/** Projects existing feature sessions into one admission rule; it owns no second session state. */
export function useNodeOperationGate() {
  const imageEdit = useImageEditSession();
  const imageRemoval = useImageRemovalSession();
  const imageComment = useImageCommentSession();
  const canvas = useCanvasEditorSession();
  const space = useSpaceStore();
  const drafts = useNodeOperationDrafts();
  const active = computed<ActiveNodeOperation | undefined>(() => {
    if (imageEdit.active) return { kind: "image-edit", nodeId: imageEdit.active.nodeId };
    if (imageRemoval.active) return { kind: "image-removal", nodeId: imageRemoval.active.nodeId };
    if (imageComment.activeNodeId) return { kind: "image-comment", nodeId: imageComment.activeNodeId };
    if (canvas.activeTextEditorNodeId.value)
      return { kind: "text-edit", nodeId: canvas.activeTextEditorNodeId.value };
    if (space.focusedEditorId) return { kind: "document-edit", nodeId: space.focusedEditorId };
    if (space.videoEditorNodeId) return { kind: "video-edit", nodeId: space.videoEditorNodeId };
    if (drafts.active) return { kind: "node-draft", nodeId: drafts.active.nodeId };
    if (canvas.directManipulation.value) return { kind: "canvas-gesture" };
    return undefined;
  });

  function allows(effect: NodeOperationEffect) {
    return effect === "inspect" || !active.value;
  }

  // The canvas gesture owns its completion callback while direct manipulation is projected.
  function allowsGestureCommit() {
    return !active.value || active.value.kind === "canvas-gesture";
  }

  return { active, allows, allowsGestureCommit };
}
