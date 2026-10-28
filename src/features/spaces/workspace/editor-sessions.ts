import { isSpaceEditorNode, type SpaceEditorNode } from "../canvas/node-capabilities";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import type { SpaceEditorTab } from "./workbench-tab";

/** Whether the `kind` editor edits `node`; an editor tab exists only for content its editor accepts. */
export function acceptsEditorContent(kind: SpaceEditorTab["kind"], node: SpaceCanvasNode) {
  return kind === "document" ? isSpaceEditorNode(node) : node.data.kind === "video";
}

export function createEditorSessions() {
  const documents = useDocumentSessionStore();
  const videos = useVideoSessionStore();
  return {
    open(tab: SpaceEditorTab, node: SpaceCanvasNode) {
      if (!acceptsEditorContent(tab.kind, node))
        throw new Error(`Content does not support the ${tab.kind} editor`);
      if (tab.kind === "document") documents.open(tab, node as SpaceEditorNode);
      else videos.open(tab, node as SpaceCanvasNode<SpaceVideoPayload>);
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
