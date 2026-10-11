import type { SpaceSessionContext } from "./space-session-context";
import { transitionSpaceSurface } from "./space-transition";

export function createEditorSurfaceActions(context: SpaceSessionContext) {
  async function openEditor(kind: "document" | "video", nodeId: string) {
    const current = context.surface.value;
    if (current.kind === kind && current.nodeId === nodeId) return;
    await transitionSpaceSurface(context, { kind, nodeId });
  }

  return {
    openDocumentEditor: (nodeId: string) => openEditor("document", nodeId),
    openVideoEditor: (nodeId: string) => openEditor("video", nodeId),
    returnToCanvas: () =>
      context.surface.value.kind === "canvas"
        ? Promise.resolve()
        : transitionSpaceSurface(context, { kind: "canvas" }),
    // Called only after the video owner's Save/Discard decision has completed.
    closeVideoEditor: () => {
      if (context.surface.value.kind === "video") context.surface.value = { kind: "canvas" };
    },
  };
}
