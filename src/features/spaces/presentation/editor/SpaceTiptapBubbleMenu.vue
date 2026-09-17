<script setup lang="ts">
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";

import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import SpaceTiptapInlineMarks from "./SpaceTiptapInlineMarks.vue";
import SpaceTiptapLinkControl from "./SpaceTiptapLinkControl.vue";
import SpaceTiptapTextStyleMenu from "./SpaceTiptapTextStyleMenu.vue";
import SpaceTiptapWritingMenu from "./SpaceTiptapWritingMenu.vue";

const props = defineProps<{ editor: Editor }>();
</script>

<template>
  <BubbleMenu :editor="editor">
    <div
      class="z-50 flex items-center gap-0.5 rounded-dialog border border-border bg-overlay p-1 shadow-popover"
    >
      <SpaceTiptapWritingMenu :editor="props.editor" compact />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapTextStyleMenu :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapInlineMarks :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapLinkControl :editor="props.editor" />
    </div>
  </BubbleMenu>
</template>
