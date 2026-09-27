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
  { name: "yellow", color: "var(--color-content-yellow-subtle)" },
  { name: "green", color: "var(--color-content-green-subtle)" },
  { name: "blue", color: "var(--color-content-blue-subtle)" },
  { name: "pink", color: "var(--color-content-pink-subtle)" },
] as const;
</script>

<template>
  <SpaceTiptapInlineMarks :editor="props.editor" />

  <UiDropdownMenu align="start" :side-offset="5" class="min-w-44">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-11 items-center justify-center gap-0.5 rounded-md px-1.5 text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary data-[state=open]:bg-surface-control"
        :class="editor.isActive('highlight') && 'bg-(--knovum-sidebar-item-bg-selected) text-fg-primary'"
        :aria-label="t('spaces.richtext.highlight')"
        :title="t('spaces.richtext.highlight')"
      >
        <Highlighter class="size-3.5" :stroke-width="1.6" />
        <ChevronDown class="size-3" :stroke-width="1.6" />
      </button>
    </template>

    <UiDropdownMenuItem
      v-for="option in highlights"
      :key="option.name"
      @select="editor.chain().focus().setHighlight({ color: option.color }).run()"
    >
      <span
        class="size-3.5 rounded-md ring-1 ring-stroke-default"
        :style="{ backgroundColor: option.color }"
      />
      <span class="min-w-0 flex-1 truncate">{{ t(`spaces.richtext.highlightColor.${option.name}`) }}</span>
      <Check
        v-if="editor.isActive('highlight', { color: option.color })"
        class="ml-auto size-3 text-fg-accent"
      />
    </UiDropdownMenuItem>
    <UiSeparator class="my-1" />
    <UiDropdownMenuItem @select="editor.chain().focus().unsetHighlight().run()">
      <span class="size-3.5 rounded-md bg-surface-canvas ring-1 ring-stroke-default" />
      <span class="min-w-0 flex-1 truncate">{{ t("spaces.richtext.highlightColor.clear") }}</span>
    </UiDropdownMenuItem>
  </UiDropdownMenu>

  <SpaceTiptapLinkControl :editor="props.editor" />
  <UiIconButton
    :label="t('spaces.richtext.superscript')"
    size="sm"
    :active="editor.isActive('superscript')"
    @click="editor.chain().focus().toggleSuperscript().run()"
  >
    <Superscript class="size-3.5" :stroke-width="1.6" />
  </UiIconButton>
  <UiIconButton
    :label="t('spaces.richtext.subscript')"
    size="sm"
    :active="editor.isActive('subscript')"
    @click="editor.chain().focus().toggleSubscript().run()"
  >
    <Subscript class="size-3.5" :stroke-width="1.6" />
  </UiIconButton>
</template>
