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
    :label="t('spaces.richtext.bold')"
    size="sm"
    :active="editor.isActive('bold')"
    @click="editor.chain().focus().toggleBold().run()"
  >
    <Bold class="size-3.5" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('spaces.richtext.italic')"
    size="sm"
    :active="editor.isActive('italic')"
    @click="editor.chain().focus().toggleItalic().run()"
  >
    <Italic class="size-3.5" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('spaces.richtext.underline')"
    size="sm"
    :active="editor.isActive('underline')"
    @click="editor.chain().focus().toggleUnderline().run()"
  >
    <Underline class="size-3.5" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('spaces.richtext.strike')"
    size="sm"
    :active="editor.isActive('strike')"
    @click="editor.chain().focus().toggleStrike().run()"
  >
    <Strikethrough class="size-3.5" :stroke-width="1.7" />
  </UiIconButton>
  <UiIconButton
    :label="t('spaces.richtext.inlineCode')"
    size="sm"
    :active="editor.isActive('code')"
    @click="editor.chain().focus().toggleCode().run()"
  >
    <Code2 class="size-3.5" :stroke-width="1.6" />
  </UiIconButton>
</template>
