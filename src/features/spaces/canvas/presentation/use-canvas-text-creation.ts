import { useI18n } from "vue-i18n";

import { createSpaceNotePayload, createSpaceTextPayload } from "../../document-editing/tiptap-content";
import { useSpaceStore } from "../../session/space-store";
import { useCanvasToolStore } from "../canvas-tool-store";
import type { CanvasPoint } from "../contracts/canvas-editor-port";

export function useCanvasTextCreation() {
  const store = useSpaceStore();
  const tools = useCanvasToolStore();
  const { t } = useI18n();

  return async (kind: "note" | "text", point: CanvasPoint) => {
    const spaceId = store.activeSpaceId;
    const title = t(kind === "note" ? "spaces.editor.note.new" : "spaces.editor.text.new");
    const payload = kind === "note" ? createSpaceNotePayload(title) : createSpaceTextPayload(title);
    const [node] = await store.addContent(payload, point, { reveal: false });
    if (!node) throw new Error("Canvas text creation did not produce a node");
    if (store.activeSpaceId !== spaceId || tools.tool !== kind) return;
    tools.resetTool();
    return node.id;
  };
}
