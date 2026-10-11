<script setup lang="ts">
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";

import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceTiptapInlineMarks from "./SpaceTiptapInlineMarks.vue";
import SpaceTiptapLinkControl from "./SpaceTiptapLinkControl.vue";
import SpaceTiptapTextStyleMenu from "./SpaceTiptapTextStyleMenu.vue";
import SpaceTiptapWritingMenu from "./SpaceTiptapWritingMenu.vue";

const props = defineProps<{ editor: Editor; writing: SpaceTiptapWritingTools }>();
</script>

<template>
  <BubbleMenu :editor="editor">
    <div
      class="z-50 flex items-center gap-0.5 rounded-xl border border-stroke-default bg-surface-overlay p-1 shadow-popover"
    >
      <SpaceTiptapWritingMenu :writing="props.writing" compact />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapTextStyleMenu :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapInlineMarks :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapLinkControl :editor="props.editor" />
    </div>
  </BubbleMenu>
</template>
