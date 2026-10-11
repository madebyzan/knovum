import { isSpaceEditorNode } from "../canvas/node-capabilities";
import { useDocumentSessionStore } from "../editor/document-session-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import type { SpaceEditorTab } from "./work-items";

export function createEditorSessions() {
  const documents = useDocumentSessionStore();
  const videos = useVideoSessionStore();
  return {
    open(tab: SpaceEditorTab, node: SpaceCanvasNode) {
      if (tab.kind === "document" && isSpaceEditorNode(node)) documents.open(tab, node);
      else if (tab.kind === "video" && node.data.kind === "video")
        videos.open(tab, node as SpaceCanvasNode<SpaceVideoPayload>);
      else throw new Error("Editor content is unavailable");
    },
    canDiscard(tab: SpaceEditorTab) {
      return tab.kind === "document"
        ? Boolean(documents.sessions.get(tab.id)?.error.value)
        : Boolean(videos.sessions.get(tab.id)?.error.value);
    },
    release(tab: SpaceEditorTab) {
      if (tab.kind === "document") documents.release(tab.id);
      else videos.release(tab.id);
    },
  };
}
