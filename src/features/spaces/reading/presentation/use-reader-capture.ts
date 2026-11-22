import { writeText } from "tauri-plugin-clipboard-api";
import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { CanvasPoint } from "../../canvas/contracts/canvas-editor-port";
import { currentCanvasEditor, getCanvasEditorSession } from "../../canvas/editor-session";
import { acceptsPassage, addPassageNode, appendPassage } from "../../canvas/passage-capture";
import { noteFromPlainText } from "../../canvas/text-import";
import { useCanvasSelectionProjection } from "../../canvas/use-canvas-selection-projection";
import { useContextDock } from "../../canvas/use-context-dock";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { quotedFollowUp } from "../../conversation/answer-capture";
import { useConversationSessionStore } from "../../conversation/conversation-session-store";
import { createSourceEvidence } from "../../document-editing/knowledge-capture";
import { knowledgeLocationLabel } from "../../knowledge/model/location-label";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { truncateUtf16 } from "../../model/text";
import { useSpaceStore } from "../../session/space-store";
import { buildSourceReaderLocator } from "../source-reader-target";
import type { SourceReaderPassage } from "./use-reader-selection";

const TITLE_UNITS = 40;

export interface AppendNoteTarget {
  id: string;
  title: string;
}

export type ReaderDropTarget =
  | { kind: "create"; point: CanvasPoint }
  | { kind: "append"; nodeId: string; title: string };

export function useReaderCapture(sourceNode: Readonly<Ref<SpaceCanvasNode | undefined>>) {
  const store = useSpaceStore();
  const gate = useNodeOperationGate();
  const dock = useContextDock();
  const conversationSession = useConversationSessionStore();
  const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
  const { t } = useI18n();

  const canSave = computed(() => Boolean(store.activeSpaceId && sourceNode.value) && gate.allows("create"));
  const canAppend = computed(() => Boolean(store.activeSpaceId && sourceNode.value) && gate.allows("edit"));
  const selectedAppendTarget = computed<AppendNoteTarget | undefined>(() => {
    if (!canAppend.value) return undefined;
    const [node, ...others] = selectedNodes.value;
    if (!node || others.length || node.id === sourceNode.value?.id || !acceptsPassage(node.data))
      return undefined;
    return { id: node.id, title: truncateUtf16(node.data.title, TITLE_UNITS) };
  });
  const existingNoteTargets = computed<readonly AppendNoteTarget[]>(() => {
    if (!canAppend.value) return [];
    const sourceId = sourceNode.value?.id;
    const derivedIds = new Set(
      store.relations
        .filter(
          (rel) => rel.kind === "derives-from" && rel.status !== "invalidated" && rel.targetId === sourceId,
        )
        .map((rel) => rel.sourceId),
    );
    return store.nodes
      .filter((node) => !node.hidden && node.id !== sourceId && acceptsPassage(node.data))
      .toReversed()
      .toSorted((left, right) => Number(derivedIds.has(right.id)) - Number(derivedIds.has(left.id)))
      .slice(0, 16)
      .map((node) => ({ id: node.id, title: truncateUtf16(node.data.title, TITLE_UNITS) }));
  });

  async function runCapture(
    task: () => Promise<SpaceCanvasNode | undefined>,
    formatNotice: (nodeTitle: string) => string,
  ) {
    try {
      const created = await task();
      if (!created) return;
      const label = truncateUtf16(created.data.title, TITLE_UNITS);
      toast.success(formatNotice(label), {
        action: {
          label: t("spaces.conversation.capture.show"),
          onClick: () => {
            store.requestReveal([created.id], "focus");
            getCanvasEditorSession().selectNodes([created.id]);
          },
        },
      });
    } catch (caught) {
      toast.error(getErrorMessage(caught));
    }
  }

  return {
    canSave,
    selectedAppendTarget,
    existingNoteTargets,
    async createNote(passage: SourceReaderPassage, point?: CanvasPoint) {
      const source = sourceNode.value;
      if (!canSave.value || !source) return;
      const locator = buildSourceReaderLocator(source.data, passage.location, passage.text);
      const payload = noteFromPlainText(passage.text, t("spaces.editor.note.new"));
      await runCapture(
        () => addPassageNode(payload, [{ nodeId: source.id, locator, quote: passage.text }], point),
        () => t("spaces.conversation.capture.saved"),
      );
    },
    async createEvidence(passage: SourceReaderPassage) {
      const source = sourceNode.value;
      if (!canSave.value || !source) return;
      const locator = buildSourceReaderLocator(source.data, passage.location, passage.text);
      await runCapture(
        () =>
          createSourceEvidence({
            target: { nodeId: source.id, nodeTitle: source.data.title, locator, text: passage.text },
            title: t("spaces.inspector.excerpts.title", { title: source.data.title }),
            sourceLine: t("spaces.inspector.excerpts.line", {
              title: source.data.title,
              location: knowledgeLocationLabel(locator),
            }),
            rationale: t("spaces.conversation.evidence.sourceRationale"),
          }),
        () => t("spaces.conversation.capture.saved"),
      );
    },
    async appendToNode(passage: SourceReaderPassage, targetId?: string) {
      const source = sourceNode.value;
      const resolvedId = targetId ?? selectedAppendTarget.value?.id;
      if (!canAppend.value || !source || !resolvedId) return;
      const locator = buildSourceReaderLocator(source.data, passage.location, passage.text);
      await runCapture(
        () => appendPassage(resolvedId, passage.text, [{ nodeId: source.id, locator, quote: passage.text }]),
        (title) => t("spaces.conversation.capture.appended", { title }),
      );
    },
    askAboutPassage(passage: SourceReaderPassage) {
      const source = sourceNode.value;
      if (!source) return;
      dock.openAsk();
      conversationSession.addNodeAttachments([source.id]);
      conversationSession.prompt = quotedFollowUp(passage.text, conversationSession.prompt);
    },
    async copyPassage(passage: SourceReaderPassage) {
      await writeText(passage.text).catch((error: unknown) => toast.error(getErrorMessage(error)));
    },
    dropTargetAt(clientPoint: CanvasPoint): ReaderDropTarget | undefined {
      const editor = currentCanvasEditor();
      const point = editor?.canvasPointAt(clientPoint);
      if (!point) return undefined;
      if (canAppend.value) {
        const hit = editor
          ?.getDocument()
          .nodes.toReversed()
          .find(
            ({ nodeId, frame }) =>
              point.x >= frame.x &&
              point.x <= frame.x + frame.width &&
              point.y >= frame.y &&
              point.y <= frame.y + frame.height &&
              nodeId !== sourceNode.value?.id,
          );
        const node = hit
          ? store.nodes.find((n) => n.id === hit.nodeId && !n.hidden && acceptsPassage(n.data))
          : undefined;
        if (node)
          return { kind: "append", nodeId: node.id, title: truncateUtf16(node.data.title, TITLE_UNITS) };
      }
      return canSave.value ? { kind: "create", point } : undefined;
    },
  };
}
