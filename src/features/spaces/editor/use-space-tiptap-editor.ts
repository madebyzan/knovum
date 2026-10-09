import { useEditor } from "@tiptap/vue-3";
import { useDebounceFn } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, nextTick, onBeforeUnmount, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { isSpaceEditorNode, type SpaceEditorNode } from "../canvas/node-capabilities";
import { useContextDock } from "../canvas/use-context-dock";
import { loadGroundedDocumentMetadata } from "../knowledge-derivations/document/metadata";
import { findGroundedCitation } from "../knowledge/model/grounded-citation";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore as useStore } from "../session/space-store";
import { createSpaceTiptapExtensions } from "./tiptap-extensions";
import { revealKnowledgeInEditor } from "./tiptap-locator";
import { spaceTiptapHeadings } from "./tiptap-outline";
import { createEditingState, persistSpaceTiptapContent, type EditingState } from "./tiptap-persistence";
import { persistTiptapSession, registerTiptapSessionParticipant } from "./tiptap-session-participant";
import { createTiptapTextStatistics } from "./tiptap-text-statistics";
import { useHeadingNavigation } from "./use-heading-navigation";

export function useSpaceTiptapEditor() {
  const store = useStore();
  const { editorLocateRequest, focusedEditorId, nodes } = storeToRefs(store);
  const { t } = useI18n();
  const dock = useContextDock();
  const state = createEditingState();
  const saveStatus = ref<"saved" | "saving" | "unsaved">("saved");
  const editorNode = computed(() => findEditorNode(nodes.value, focusedEditorId.value));
  initializeEditingState(state, editorNode.value);
  const initialContent = editorContent(editorNode.value);
  const statistics = createTiptapTextStatistics();
  const headings = shallowRef<ReturnType<typeof spaceTiptapHeadings>>([]);
  const derive = useDebounceFn(() => {
    const instance = editor.value;
    if (!instance || instance.isDestroyed) return;
    statistics.update(instance.getText());
    headings.value = spaceTiptapHeadings(instance);
  }, 200);

  const persist = async () => {
    saveStatus.value = "saving";
    try {
      await persistSpaceTiptapContent({ store, nodes: nodes.value, state, editor: editor.value });
      saveStatus.value = "saved";
    } catch (error) {
      saveStatus.value = "unsaved";
      throw error;
    }
  };

  const persistInBackground = () => {
    void persist().catch((error) => toast.error(getErrorMessage(error)));
  };

  const autoSave = useDebounceFn(persistInBackground, 1000);

  const scheduleAutoSave = () => {
    saveStatus.value = "unsaved";
    autoSave();
  };

  const editor = useEditor({
    extensions: createSpaceTiptapExtensions(
      t("spaces.editor.placeholder"),
      (error) =>
        toast.error(t("spaces.richtext.image.uploadFailed"), { description: getErrorMessage(error) }),
      async (index) => {
        const node = editorNode.value;
        if (!node || node.data.kind !== "markdown") return;
        const metadata = await loadGroundedDocumentMetadata(node.id);
        const citation = findGroundedCitation(metadata?.citations, index);
        if (!citation || citation.state !== "current" || !citation.chunkId) return;
        dock.openSource({
          nodeId: citation.nodeId,
          nodeTitle: citation.nodeTitle,
          chunkId: citation.chunkId,
          locator: citation.locator,
          text: citation.quote,
        });
      },
    ),
    content: initialContent.content,
    ...(initialContent.contentType === undefined ? {} : { contentType: initialContent.contentType }),
    editorProps: { attributes: { class: "space-tiptap-prose", spellcheck: "true" } },
    onCreate: () => void derive(),
    onUpdate: () => {
      state.revision.value += 1;
      void derive();
      scheduleAutoSave();
    },
    onSelectionUpdate: ({ editor: instance }) => (state.selection.value = instance.state.selection.from),
  });

  watch(
    [editorLocateRequest, editorNode, editor],
    ([request, node, instance]) => {
      if (!request || request.nodeId !== node?.id || !instance) return;
      void nextTick(() => revealKnowledgeInEditor(instance, request.locator));
    },
    { immediate: true },
  );

  const outline = useHeadingNavigation(
    () => editor.value,
    () => headings.value,
  );
  const { activeHeadingId, revealHeading, scrollRoot, syncActiveHeading } = outline;
  const editorLabel = computed(() =>
    editorNode.value?.data.kind === "note" ? t("spaces.nodeTypes.note") : t("spaces.nodeTypes.markdown"),
  );

  async function close() {
    try {
      await persistTiptapSession(persist, () => {
        state.loadedId = undefined;
        focusedEditorId.value = undefined;
      });
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  const detachSessionCommit = registerTiptapSessionParticipant({
    cancelAutoSave: autoSave.cancel,
    editor: () => editor.value,
    persist,
  });

  onBeforeUnmount(() => {
    detachSessionCommit();
    outline.dispose();
    derive.cancel();
    statistics.dispose();
  });

  return {
    activeHeadingId,
    characterCount: statistics.characterCount,
    close,
    editorLabel,
    editorNode,
    editor,
    headings,
    persist,
    revealHeading,
    saveStatus,
    scrollRoot,
    syncActiveHeading,
    title: state.title,
    wordCount: statistics.wordCount,
  };
}

function findEditorNode(nodes: readonly SpaceCanvasNode[], id?: string) {
  return nodes.find((node): node is SpaceEditorNode => node.id === id && isSpaceEditorNode(node));
}

function initializeEditingState(state: EditingState, node: SpaceEditorNode | undefined) {
  if (!node) return;
  state.loadedId = node.id;
  state.title.value = node.data.title;
  if (node.data.kind === "markdown") {
    state.markdown.value = node.data.text;
    if (node.data.tiptapContent) state.tiptapContent.value = node.data.tiptapContent;
    return;
  }
  state.tiptapContent.value = node.data.tiptapContent;
  state.plainText.value = node.data.plainText;
}

function editorContent(node: SpaceEditorNode | undefined): {
  content: string | object;
  contentType?: "markdown";
} {
  if (!node) return { content: "", contentType: "markdown" };
  if (node.data.kind === "note") return { content: node.data.tiptapContent };
  if (node.data.tiptapContent) return { content: node.data.tiptapContent };
  return { content: node.data.text, contentType: "markdown" };
}
