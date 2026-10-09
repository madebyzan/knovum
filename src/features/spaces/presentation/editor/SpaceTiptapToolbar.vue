<script setup lang="ts">
import { AlignCenter, AlignJustify, AlignLeft, AlignRight, Redo2, Undo2 } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type { SpaceTiptapWritingTools } from "../../editor/use-space-tiptap-writing-tools";
import SpaceTiptapBlockControls from "./SpaceTiptapBlockControls.vue";
import SpaceTiptapInsertControls from "./SpaceTiptapInsertControls.vue";
import SpaceTiptapMarkControls from "./SpaceTiptapMarkControls.vue";
import SpaceTiptapWritingMenu from "./SpaceTiptapWritingMenu.vue";

defineProps<{ editor: Editor; writing: SpaceTiptapWritingTools }>();
const { t } = useI18n();
const alignments = [
  { value: "left", icon: AlignLeft },
  { value: "center", icon: AlignCenter },
  { value: "right", icon: AlignRight },
  { value: "justify", icon: AlignJustify },
] as const;
</script>

<template>
  <div class="flex min-w-0 items-center px-2 py-1.5">
    <div class="flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto overscroll-x-contain">
      <UiIconButton
        :label="t('common.undo')"
        shortcut="$mod+Z"
        size="sm"
        :disabled="!editor.can().undo()"
        @click="editor.chain().focus().undo().run()"
      >
        <Undo2 class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiIconButton
        :label="t('common.redo')"
        shortcut="$mod+Shift+Z"
        size="sm"
        :disabled="!editor.can().redo()"
        @click="editor.chain().focus().redo().run()"
      >
        <Redo2 class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-1 !h-5" />
      <SpaceTiptapBlockControls :editor="editor" />
      <UiSeparator orientation="vertical" class="mx-1 !h-5" />
      <SpaceTiptapMarkControls :editor="editor" />
      <UiSeparator orientation="vertical" class="mx-1 !h-5" />
      <UiIconButton
        v-for="alignment in alignments"
        :key="alignment.value"
        :label="t(`spaces.richtext.align.${alignment.value}`)"
        size="sm"
        :active="editor.isActive({ textAlign: alignment.value })"
        @click="editor.chain().focus().setTextAlign(alignment.value).run()"
      >
        <component :is="alignment.icon" class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-1 !h-5" />
      <SpaceTiptapInsertControls :editor="editor" />
    </div>
    <UiSeparator orientation="vertical" class="mx-1 !h-5" />
    <div class="shrink-0 bg-surface-chrome whitespace-nowrap">
      <SpaceTiptapWritingMenu :writing="writing" />
    </div>
  </div>
</template>
