import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { noteFromPlainText } from "../canvas/text-import";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import { markdownFromAnswer } from "./answer-capture";

/** Puts what an answer says on the canvas, where the person can arrange, edit and connect it. */
export function useAnswerCapture() {
  const store = useSpaceStore();
  const gate = useNodeOperationGate();
  const { t } = useI18n();
  const canSave = computed(() => Boolean(store.activeSpaceId) && gate.allows("create"));

  async function save(payload: SpaceNodePayload) {
    if (!canSave.value) return;
    try {
      await store.addContent(payload);
      toast.success(t("spaces.conversation.capture.saved"));
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  return {
    canSave,
    saveAnswer: (markdown: string) =>
      save(markdownFromAnswer(markdown, t("spaces.editor.markdown.untitled"))),
    saveSelection: (text: string) => save(noteFromPlainText(text, t("spaces.editor.note.new"))),
  };
}
