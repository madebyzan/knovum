import { isSpaceEditorNode } from "../canvas/node-capabilities";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import type { SpaceEditorTab } from "./workbench-tab";

export function createEditorSessions() {
  const documents = useDocumentSessionStore();
  const videos = useVideoSessionStore();
  return {
    open(tab: SpaceEditorTab, node: SpaceCanvasNode) {
      if (tab.kind === "document") {
        if (!isSpaceEditorNode(node)) throw new Error("Content does not support the document editor");
        documents.open(tab, node);
        return;
      }
      if (node.data.kind !== "video") throw new Error("Content does not support the video editor");
      videos.open(tab, node as SpaceCanvasNode<SpaceVideoPayload>);
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
