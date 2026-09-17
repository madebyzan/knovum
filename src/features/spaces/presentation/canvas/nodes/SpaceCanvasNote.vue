<script setup lang="ts">
import { EditorContent } from "@tiptap/vue-3";
import { toRef } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasTextPayload } from "../../../model/nodes/capabilities";
import type { SpaceNotePayload } from "../../../model/nodes/node-payload";
import SpaceCanvasRichText from "./SpaceCanvasRichText.vue";
import SpaceCanvasTextSelectionBar from "./SpaceCanvasTextSelectionBar.vue";
import { useCanvasTextEditor } from "./use-canvas-text-editor";

const props = defineProps<{
  id: string;
  data: SpaceNotePayload;
  interactive: boolean;
  verticalClass: string;
  fontSize: number;
  textAlign: "left" | "center" | "right";
  persist: (id: string, data: SpaceCanvasTextPayload) => Promise<void>;
}>();
const { t } = useI18n();
const placeholder = t("spaces.editor.notePlaceholder");
const { beginEditing, cancelEditing, editing, editor, save } = useCanvasTextEditor({
  id: props.id,
  data: toRef(props, "data"),
  interactive: toRef(props, "interactive"),
  placeholder,
  defaultTitle: t("spaces.editor.newNote"),
  persist: props.persist,
});
</script>

<template>
  <div
    class="flex size-full min-h-0 flex-col overflow-hidden p-4 leading-relaxed"
    :class="[verticalClass, editing ? 'pointer-events-auto' : 'select-none']"
    :tabindex="interactive && !data.locked ? 0 : undefined"
    @dblclick.stop="beginEditing"
    @keydown.enter="beginEditing"
  >
    <SpaceCanvasTextSelectionBar
      v-if="editor"
      :editor="editor"
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
      :content="data.tiptapContent"
      :empty="data.plainText.length === 0"
      kind="note"
      :placeholder="placeholder"
    />
  </div>
</template>
