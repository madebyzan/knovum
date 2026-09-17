<script setup lang="ts">
import { Bold, Code2, Italic, Strikethrough, Underline } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

defineProps<{ editor: Editor }>();
const { t } = useI18n();
</script>

<template>
  <UiIconButton
    :label="t('editor.bold')"
    size="sm"
    :active="editor.isActive('bold')"
    @click="editor.chain().focus().toggleBold().run()"
  >
    <Bold class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('editor.italic')"
    size="sm"
    :active="editor.isActive('italic')"
    @click="editor.chain().focus().toggleItalic().run()"
  >
    <Italic class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('editor.underline')"
    size="sm"
    :active="editor.isActive('underline')"
    @click="editor.chain().focus().toggleUnderline().run()"
  >
    <Underline class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('editor.strike')"
    size="sm"
    :active="editor.isActive('strike')"
    @click="editor.chain().focus().toggleStrike().run()"
  >
    <Strikethrough class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('editor.inlineCode')"
    size="sm"
    :active="editor.isActive('code')"
    @click="editor.chain().focus().toggleCode().run()"
  >
    <Code2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
  </UiIconButton>
</template>
