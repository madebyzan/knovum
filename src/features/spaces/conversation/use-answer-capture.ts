import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CanvasPoint } from "../canvas/contracts/canvas-editor-port";
import { currentCanvasEditor, getCanvasEditorSession } from "../canvas/editor-session";
import { acceptsPassage, addPassageNode, appendPassage } from "../canvas/passage-capture";
import { noteFromPlainText } from "../canvas/text-import";
import { useCanvasSelectionProjection } from "../canvas/use-canvas-selection-projection";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import type { GroundedCitation } from "../knowledge/model/schema/grounded-citation";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { truncateUtf16 } from "../model/text";
import { useSpaceStore } from "../session/space-store";
import { markdownFromAnswer, type AnswerPassage } from "./answer-capture";

const TITLE_UNITS = 40;

/**
 * Puts what an answer says on the canvas, where the person can arrange, edit and connect it. What
 * lands there is linked to the canvas items the answer cited for it.
 */
export function useAnswerCapture(citations: MaybeRefOrGetter<readonly GroundedCitation[]>) {
  const store = useSpaceStore();
  const gate = useNodeOperationGate();
  const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
  const { t } = useI18n();
  const canSave = computed(() => Boolean(store.activeSpaceId) && gate.allows("create"));
  /** The one note or Markdown item selected on the canvas: a passage can go on the end of it. */
  const appendTarget = computed(() => {
    const [node, ...others] = selectedNodes.value;
    if (!node || others.length || !acceptsPassage(node.data) || !gate.allows("edit")) return undefined;
    return { id: node.id, title: truncateUtf16(node.data.title, TITLE_UNITS) };
  });

  function sources(indexes?: readonly number[]) {
    return toValue(citations).flatMap((citation) =>
      citation.state === "current" && (!indexes || indexes.includes(citation.index)) ? [citation.nodeId] : [],
    );
  }

  function show(nodeId: string) {
    store.requestReveal([nodeId], "focus");
    getCanvasEditorSession().selectNodes([nodeId]);
  }

  async function report(
    work: () => Promise<SpaceCanvasNode | undefined>,
    message: (title: string) => string,
  ) {
    try {
      const node = await work();
      if (!node) return;
      toast.success(message(truncateUtf16(node.data.title, TITLE_UNITS)), {
        action: { label: t("spaces.conversation.capture.show"), onClick: () => show(node.id) },
      });
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  return {
    canSave,
    appendTarget,
    async saveAnswer(markdown: string) {
      if (!canSave.value) return;
      const payload = markdownFromAnswer(markdown, t("spaces.editor.markdown.untitled"));
      await report(
        () => addPassageNode(payload, sources()),
        () => t("spaces.conversation.capture.saved"),
      );
    },
    /** Saves the passage as a note at `point`, or beside the items it cites. */
    async saveSelection(passage: AnswerPassage, point?: CanvasPoint) {
      if (!canSave.value) return;
      const payload = noteFromPlainText(passage.text, t("spaces.editor.note.new"));
      await report(
        () => addPassageNode(payload, sources(passage.citations), point),
        () => t("spaces.conversation.capture.saved"),
      );
    },
    async appendSelection(passage: AnswerPassage) {
      const target = appendTarget.value;
      if (!target) return;
      await report(
        () => appendPassage(target.id, passage.text, sources(passage.citations)),
        (title) => t("spaces.conversation.capture.appended", { title }),
      );
    },
    /** The canvas point under a viewport point, where a dragged passage would land. */
    canvasPointAt: (point: CanvasPoint) =>
      canSave.value ? currentCanvasEditor()?.canvasPointAt(point) : undefined,
  };
}
