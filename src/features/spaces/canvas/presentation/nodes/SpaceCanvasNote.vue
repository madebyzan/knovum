<script setup lang="ts">
import { EditorContent } from "@tiptap/vue-3";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasTextPayload } from "../../../model/nodes/capabilities";
import type { SpaceNotePayload } from "../../../model/nodes/node-payload";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import { CANVAS_TEXT_SURFACE_ATTRIBUTE } from "./canvas-text-surface";
import SpaceCanvasEditorIndicator from "./SpaceCanvasEditorIndicator.vue";
import SpaceCanvasRichText from "./SpaceCanvasRichText.vue";
import SpaceCanvasTextSelectionBar from "./SpaceCanvasTextSelectionBar.vue";
import { projectTiptapPreview } from "./tiptap-preview-projection";
import { useCanvasTextEditor } from "./use-canvas-text-editor";

const props = defineProps<{
  id: string;
  data: SpaceNotePayload;
  interactive: boolean;
  presentationContext: CanvasNodePresentationContext;
  verticalClass: string;
  fontSize: number;
  textAlign: "left" | "center" | "right";
  persist: (nodeId: string, payload: SpaceCanvasTextPayload) => Promise<void>;
}>();
const { t } = useI18n();
const placeholder = computed(() => t("spaces.editor.note.placeholder"));
const previewContent = computed(() =>
  projectTiptapPreview(props.data.tiptapContent, props.presentationContext),
);
const { beginEditing, cancelEditing, editing, editor, save } = useCanvasTextEditor({
  id: props.id,
  data: toRef(props, "data"),
  interactive: toRef(props, "interactive"),
  persist: props.persist,
});
</script>

<template>
  <div
    class="relative flex size-full min-h-0 flex-col overflow-hidden p-4 leading-relaxed"
    :class="[verticalClass, editing ? 'pointer-events-auto' : 'select-none']"
    v-bind="{ [CANVAS_TEXT_SURFACE_ATTRIBUTE]: id }"
    :tabindex="interactive ? 0 : undefined"
    @dblclick.stop="beginEditing"
    @keydown.enter="beginEditing"
  >
    <SpaceCanvasEditorIndicator :node-id="id" class="absolute top-1 right-1" />
    <SpaceCanvasTextSelectionBar
      v-if="editor"
      :node-id="id"
      :editor="editor"
      kind="note"
      :default-font-size="fontSize"
      :default-alignment="textAlign"
      :typography-scale="1"
    />
    <EditorContent
      v-if="editing && editor"
      :editor="editor"
      class="max-h-full w-full overflow-auto"
      @keydown.meta.enter.prevent="save"
      @keydown.ctrl.enter.prevent="save"
      @keydown.escape.prevent="cancelEditing"
    />
    <SpaceCanvasRichText
      v-else
      :content="previewContent"
      :empty="data.plainText.length === 0"
      kind="note"
      :placeholder="placeholder"
    />
  </div>
</template>
