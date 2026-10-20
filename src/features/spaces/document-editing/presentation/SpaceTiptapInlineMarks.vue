<script setup lang="ts">
import { Bold, Code2, Italic, Strikethrough, Underline } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

type InlineMark = "bold" | "italic" | "underline" | "strike" | "code";
withDefaults(defineProps<{ editor: Editor; marks?: readonly InlineMark[] }>(), {
  marks: () => ["bold", "italic", "underline", "strike", "code"],
});
const controls = {
  bold: {
    icon: Bold,
    label: "spaces.richtext.bold",
    toggle: (editor: Editor) => editor.chain().focus().toggleBold().run(),
  },
  italic: {
    icon: Italic,
    label: "spaces.richtext.italic",
    toggle: (editor: Editor) => editor.chain().focus().toggleItalic().run(),
  },
  underline: {
    icon: Underline,
    label: "spaces.richtext.underline",
    toggle: (editor: Editor) => editor.chain().focus().toggleUnderline().run(),
  },
  strike: {
    icon: Strikethrough,
    label: "spaces.richtext.strike",
    toggle: (editor: Editor) => editor.chain().focus().toggleStrike().run(),
  },
  code: {
    icon: Code2,
    label: "spaces.richtext.inlineCode",
    toggle: (editor: Editor) => editor.chain().focus().toggleCode().run(),
  },
};
const { t } = useI18n();
</script>

<template>
  <UiIconButton
    v-for="mark in marks"
    :key="mark"
    :label="t(controls[mark].label)"
    size="sm"
    :active="editor.isActive(mark)"
    @click="controls[mark].toggle(editor)"
  >
    <component :is="controls[mark].icon" class="size-3.5" :stroke-width="mark === 'code' ? 1.6 : 1.7" />
  </UiIconButton>
</template>
