import { useI18n } from "vue-i18n";

import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasPoint } from "../../canvas/contract/canvas-editor";
import { createSpaceNotePayload, createSpaceTextPayload } from "../../editor/tiptap-content";
import { useSpaceStore } from "../../session/space-store";

export function useCanvasTextCreation() {
  const store = useSpaceStore();
  const tools = useCanvasToolStore();
  const { t } = useI18n();

  return async (kind: "note" | "text", point: CanvasPoint) => {
    const spaceId = store.activeSpaceId;
    const title = t(kind === "note" ? "spaces.editor.newNote" : "spaces.editor.newText");
    const payload = kind === "note" ? createSpaceNotePayload(title) : createSpaceTextPayload(title);
    const [node] = await store.addContent(payload, point, { reveal: false });
    if (!node) throw new Error("Canvas text creation did not produce a node");
    if (store.activeSpaceId !== spaceId || tools.tool !== kind) return;
    tools.resetTool();
    return node.id;
  };
}
