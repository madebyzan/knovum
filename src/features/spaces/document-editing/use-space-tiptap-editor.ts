import type { EditorOptions } from "@tiptap/core";
import { Editor, useEditor } from "@tiptap/vue-3";
import { useDebounceFn } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, nextTick, onBeforeUnmount, shallowRef, watch, type Ref, type ShallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useContextDock } from "../canvas/use-context-dock";
import { loadGroundedDocumentMetadata } from "../knowledge-derivations/document/metadata";
import { findGroundedCitation } from "../knowledge/model/grounded-citation";
import type { SpaceEditorPayload } from "../model/nodes/capabilities";
import { useSpaceStore } from "../session/space-store";
import type { SpaceEditorTab } from "../workspace/workbench-tab";
import { useDocumentSessionStore } from "./document-session-store";
import { isDocumentEdit } from "./tiptap-edits";
import { createSpaceTiptapExtensions } from "./tiptap-extensions";
import { revealKnowledgeInEditor } from "./tiptap-locator";
import { spaceTiptapHeadings } from "./tiptap-outline";
import { createTiptapTextStatistics } from "./tiptap-text-statistics";
import { useDocumentSave } from "./use-document-save";
import { useHeadingNavigation } from "./use-heading-navigation";

export function useSpaceTiptapEditor(
  tab: SpaceEditorTab,
  active: Ref<boolean>,
  scrollRoot: Readonly<ShallowRef<HTMLElement | null>>,
) {
  const store = useSpaceStore();
  const { editorLocateRequest } = storeToRefs(store);
  const { t } = useI18n();
  const dock = useContextDock();
  const session = useDocumentSessionStore().sessions.get(tab.id);
  if (!session) throw new Error("Document session is unavailable");
  const editorNode = session.node;
  const statistics = createTiptapTextStatistics();
  const headings = shallowRef<ReturnType<typeof spaceTiptapHeadings>>([]);
  const derive = useDebounceFn(() => {
    const instance = editor.value;
    if (!instance || instance.isDestroyed) return;
    statistics.update(instance.getText());
    headings.value = spaceTiptapHeadings(instance);
  }, 200);
  const editorOptions: Partial<EditorOptions> = {
    editable: active.value && !session.error.value,
    extensions: createSpaceTiptapExtensions(
      t("spaces.editor.placeholder"),
      (error) =>
        toast.error(t("spaces.richtext.image.uploadFailed"), { description: getErrorMessage(error) }),
      async (index) => {
        const node = editorNode.value;
        if (!active.value || node.data.kind !== "markdown") return;
        const metadata = await loadGroundedDocumentMetadata(node.id);
        if (!active.value) return;
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
    ...editorContent(editorNode.value.data),
    editorProps: { attributes: { class: "space-tiptap-prose", spellcheck: "true" } },
    onCreate: ({ editor: instance }) => {
      if (active.value) instance.commands.focus("start");
      void derive();
    },
    onUpdate: (update) => {
      if (!isDocumentEdit(update)) return;
      session.state.revision.value += 1;
      void derive();
    },
  };
  const editor = useEditor(editorOptions);
  const saving = useDocumentSave(session, editor, active);
  watch(
    [active, session.error, saving.suspended, editor],
    ([visible, error, suspended]) => {
      editor.value?.setEditable(visible && !error && !suspended);
    },
    { immediate: true },
  );
  watch(session.contentVersion, () => {
    const previous = editor.value;
    editor.value = new Editor({ ...editorOptions, ...editorContent(editorNode.value.data) });
    previous?.destroy();
    session.state.persistedRevision.value = session.state.revision.value;
    void derive();
  });
  watch(
    [editorLocateRequest, active, editor],
    ([request, visible, instance]) => {
      if (!visible || !request || request.nodeId !== tab.nodeId || !instance) return;
      void nextTick(() => {
        if (active.value) revealKnowledgeInEditor(instance, request.locator);
      });
    },
    { immediate: true },
  );
  const navigation = useHeadingNavigation(
    scrollRoot,
    () => editor.value,
    () => headings.value,
  );
  onBeforeUnmount(() => {
    derive.cancel();
    statistics.dispose();
  });
  return {
    ...navigation,
    ...saving,
    editor,
    editorNode,
    headings,
    error: session.error,
    editorLabel: computed(() =>
      t(editorNode.value.data.kind === "note" ? "spaces.nodeTypes.note" : "spaces.nodeTypes.markdown"),
    ),
    title: session.state.title,
    characterCount: statistics.characterCount,
    wordCount: statistics.wordCount,
  };
}

function editorContent(data: SpaceEditorPayload) {
  if (data.tiptapContent) return { content: data.tiptapContent };
  if (data.kind === "markdown") return { content: data.text, contentType: "markdown" as const };
  return { content: data.tiptapContent };
}
