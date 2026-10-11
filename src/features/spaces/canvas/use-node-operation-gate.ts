import { computed } from "vue";

import { useImageCommentSessionStore } from "../comments/image-comment-session-store";
import { useImageEditSessionStore } from "../image-editing/image-edit-session-store";
import { useImageRemovalSessionStore } from "../knowledge-derivations/image-removal-session-store";
import { useImageSelectionSessionStore } from "../knowledge-derivations/image-selection-session-store";
import { useNodeOperationDraftsStore } from "../session/node-operation-drafts-store";
import { useSpaceStore } from "../session/space-store";
import { getCanvasEditorSession } from "./editor-session";

export type NodeOperationEffect = "inspect" | "edit" | "structure" | "create" | "clone" | "derive" | "export";
export type ActiveNodeOperation =
  | { kind: "image-edit"; nodeId: string }
  | { kind: "image-selection"; nodeId: string }
  | { kind: "image-removal"; nodeId: string }
  | { kind: "image-comment"; nodeId: string }
  | { kind: "text-edit"; nodeId: string }
  | { kind: "document-edit"; nodeId: string }
  | { kind: "video-edit"; nodeId: string }
  | { kind: "canvas-gesture" }
  | { kind: "node-draft"; nodeId: string };

/** Projects existing feature sessions into one admission rule; it owns no second session state. */
export function useNodeOperationGate() {
  const imageEdit = useImageEditSessionStore();
  const imageSelection = useImageSelectionSessionStore();
  const imageRemoval = useImageRemovalSessionStore();
  const imageComment = useImageCommentSessionStore();
  const canvas = getCanvasEditorSession();
  const space = useSpaceStore();
  const drafts = useNodeOperationDraftsStore();
  const active = computed<ActiveNodeOperation | undefined>(() => {
    if (imageEdit.active) return { kind: "image-edit", nodeId: imageEdit.active.nodeId };
    if (imageSelection.active) return { kind: "image-selection", nodeId: imageSelection.active.nodeId };
    if (imageRemoval.active) return { kind: "image-removal", nodeId: imageRemoval.active.nodeId };
    if (imageComment.activeNodeId) return { kind: "image-comment", nodeId: imageComment.activeNodeId };
    if (canvas.activeTextEditorNodeId.value)
      return { kind: "text-edit", nodeId: canvas.activeTextEditorNodeId.value };
    if (space.surface.kind === "document") return { kind: "document-edit", nodeId: space.surface.nodeId };
    if (space.surface.kind === "video") return { kind: "video-edit", nodeId: space.surface.nodeId };
    if (drafts.active) return { kind: "node-draft", nodeId: drafts.active.nodeId };
    if (canvas.directManipulation.value) return { kind: "canvas-gesture" };
    return undefined;
  });

  function allows(effect: NodeOperationEffect) {
    return space.workbench.canvas && !canvas.transitioning.value && (effect === "inspect" || !active.value);
  }

  // The canvas gesture owns its completion callback while direct manipulation is projected.
  function allowsGestureCommit() {
    return (
      space.workbench.canvas &&
      !canvas.transitioning.value &&
      (!active.value || active.value.kind === "canvas-gesture")
    );
  }

  return { active, allows, allowsGestureCommit };
}
