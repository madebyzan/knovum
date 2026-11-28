import { isEqual } from "es-toolkit";
import type { Ref } from "vue";

import { applySpaceNodeContent } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { loadSpaceSessionProjection } from "../session/space-projection";
import { useSpaceStore } from "../session/space-store";
import { useEditorAutosave } from "../workspace/use-editor-autosave";
import type { VideoSession } from "./video-session-store";

function isStaleNodeConflict(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return message.includes("changed before this edit was saved");
}

function sameVideoEditBaseline(left: SpaceVideoPayload, right: SpaceVideoPayload): boolean {
  return (
    left.sourceRevision === right.sourceRevision && isEqual(left.videoEditProject, right.videoEditProject)
  );
}

/** A video edit is the tab's work like a document's text, so it is stored the same way. */
export function useVideoProjectSave(session: VideoSession, active: Readonly<Ref<boolean>>) {
  const store = useSpaceStore();

  function latestLocalNode(): SpaceCanvasNode<SpaceVideoPayload> {
    const base = session.node.value;
    if (store.activeSpaceId !== session.tab.spaceId) return base;
    const live = store.nodes.find((candidate) => candidate.id === base.id);
    if (!live || live.data.kind !== "video") return base;
    return sameVideoEditBaseline(live.data, base.data) ? (live as SpaceCanvasNode<SpaceVideoPayload>) : base;
  }

  async function saveAgainst(
    node: SpaceCanvasNode<SpaceVideoPayload>,
    snapshot: NonNullable<VideoSession["project"]["value"]>,
  ) {
    const payload = await store.saveNodeContentForSpace(session.tab.spaceId, {
      id: node.id,
      expectedContent: node.data,
      data: { ...node.data, videoEditProject: snapshot },
    });
    session.node.value = applySpaceNodeContent(node, payload) as SpaceCanvasNode<SpaceVideoPayload>;
    session.history.markSaved(snapshot);
  }

  return useEditorAutosave({
    tab: session.tab,
    active,
    dirty: session.history.dirty,
    saving: session.saving,
    error: session.error,
    edits: [session.project],
    write: async () => {
      const snapshot = session.project.value;
      if (!snapshot) return;
      const base = latestLocalNode();
      try {
        await saveAgainst(base, snapshot);
      } catch (error) {
        if (!isStaleNodeConflict(error)) throw error;
        const projection = await loadSpaceSessionProjection(session.tab.spaceId);
        const fresh = projection.nodes.find((candidate) => candidate.id === base.id);
        if (!fresh || fresh.data.kind !== "video") {
          session.error.value = "missing";
          throw error;
        }
        if (!sameVideoEditBaseline(fresh.data, base.data)) {
          session.error.value = "conflict";
          throw error;
        }
        await saveAgainst(fresh as SpaceCanvasNode<SpaceVideoPayload>, snapshot);
      }
    },
  });
}
