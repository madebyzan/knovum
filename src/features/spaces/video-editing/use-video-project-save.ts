import type { Ref } from "vue";

import { applySpaceNodeContent } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import { useEditorAutosave } from "../workspace/use-editor-autosave";
import type { VideoSession } from "./video-session-store";

/** A video edit is the tab's work like a document's text, so it is stored the same way. */
export function useVideoProjectSave(session: VideoSession, active: Readonly<Ref<boolean>>) {
  const store = useSpaceStore();
  return useEditorAutosave({
    tab: session.tab,
    active,
    dirty: session.history.dirty,
    saving: session.saving,
    error: session.error,
    edits: session.project,
    write: async () => {
      const node = session.node.value;
      const snapshot = session.project.value;
      if (!snapshot) return;
      const payload = await store.saveNodeContentForSpace(session.tab.spaceId, {
        id: node.id,
        expectedContent: node.data,
        data: { ...node.data, videoEditProject: snapshot },
      });
      session.node.value = applySpaceNodeContent(node, payload) as SpaceCanvasNode<SpaceVideoPayload>;
      session.history.markSaved(snapshot);
    },
  });
}
