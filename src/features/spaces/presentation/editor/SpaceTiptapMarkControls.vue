<script setup lang="ts">
import { Check, ChevronDown, Highlighter, Subscript, Superscript } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import SpaceTiptapInlineMarks from "./SpaceTiptapInlineMarks.vue";
import SpaceTiptapLinkControl from "./SpaceTiptapLinkControl.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const highlights = [
  { name: "yellow", color: "var(--knovum-editor-highlight-color-yellow)" },
  { name: "green", color: "var(--knovum-editor-highlight-color-green)" },
  { name: "blue", color: "var(--knovum-editor-highlight-color-blue)" },
  { name: "pink", color: "var(--knovum-editor-highlight-color-pink)" },
] as const;
</script>

<template>
  <SpaceTiptapInlineMarks :editor="props.editor" />

  <UiDropdownMenu align="start" :side-offset="5" class="min-w-44">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-11 items-center justify-center gap-0.5 rounded-control px-1.5 text-foreground-secondary hover:bg-control-hover hover:text-foreground data-[state=open]:bg-control-pressed"
        :class="
          editor.isActive('highlight') && 'bg-(--knovum-sidebar-item-background-selected) text-foreground'
        "
        :aria-label="t('editor.highlight')"
        :title="t('editor.highlight')"
      >
        <Highlighter class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        <ChevronDown class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />
      </button>
    </template>

    <UiDropdownMenuItem
      v-for="option in highlights"
      :key="option.name"
      @select="editor.chain().focus().setHighlight({ color: option.color }).run()"
    >
      <span class="size-3.5 rounded-control ring-1 ring-border" :style="{ backgroundColor: option.color }" />
      <span class="min-w-0 flex-1 truncate">{{ t(`editor.highlightColor.${option.name}`) }}</span>
      <Check
        v-if="editor.isActive('highlight', { color: option.color })"
        class="ml-auto size-(--knovum-icon-size-xs) text-accent"
      />
    </UiDropdownMenuItem>
    <UiSeparator class="my-1" />
    <UiDropdownMenuItem @select="editor.chain().focus().unsetHighlight().run()">
      <span class="size-3.5 rounded-control bg-canvas ring-1 ring-border" />
      <span class="min-w-0 flex-1 truncate">{{ t("editor.highlightColor.clear") }}</span>
    </UiDropdownMenuItem>
  </UiDropdownMenu>

  <SpaceTiptapLinkControl :editor="props.editor" />
  <UiIconButton
    :label="t('editor.superscript')"
    size="sm"
    :active="editor.isActive('superscript')"
    @click="editor.chain().focus().toggleSuperscript().run()"
  >
    <Superscript class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
  </UiIconButton>
  <UiIconButton
    :label="t('editor.subscript')"
    size="sm"
    :active="editor.isActive('subscript')"
    @click="editor.chain().focus().toggleSubscript().run()"
  >
    <Subscript class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
  </UiIconButton>
</template>
