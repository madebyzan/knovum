<script setup lang="ts">
import { ImagePlus } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import type { MathEditTarget } from "../tiptap-math";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceTiptapBubbleMenu from "./SpaceTiptapBubbleMenu.vue";
import SpaceTiptapDragHandle from "./SpaceTiptapDragHandle.vue";
import SpaceTiptapLinkMenu from "./SpaceTiptapLinkMenu.vue";
import SpaceTiptapMathPopover from "./SpaceTiptapMathPopover.vue";
import SpaceTiptapTableEdgeControls from "./SpaceTiptapTableEdgeControls.vue";
import SpaceTiptapTableMenu from "./SpaceTiptapTableMenu.vue";

defineProps<{
  editor: Editor;
  writing: SpaceTiptapWritingTools;
  articleHost: HTMLElement | null;
  proseHost: HTMLElement | null;
  scrollRoot: HTMLElement | null;
  mathTarget: MathEditTarget | null;
  dropActive: boolean;
  dropIndicatorTop: number | null;
}>();

const emit = defineEmits<{ askInChat: [quote?: string]; closeMath: []; convertToMath: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    v-if="dropActive && dropIndicatorTop !== null"
    data-testid="tiptap-image-drop-line"
    :style="{ top: `${dropIndicatorTop}px` }"
    class="pointer-events-none absolute inset-x-0 z-30 h-0.5 rounded-full bg-surface-accent"
  >
    <span class="absolute -top-[3px] -left-1 size-2 rounded-full bg-surface-accent" />
  </div>
  <div
    v-if="dropActive"
    :class="[
      POPOVER_SURFACE_CLASSES,
      'pointer-events-none fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 items-center gap-2 px-3 py-1.5 text-caption font-medium text-fg-primary',
    ]"
  >
    <ImagePlus class="size-3.5 text-fg-accent" :stroke-width="1.65" />{{ t("spaces.richtext.image.dropNow") }}
  </div>
  <SpaceTiptapBubbleMenu
    :editor="editor"
    :writing="writing"
    :math-open="Boolean(mathTarget)"
    @ask-in-chat="(q) => emit('askInChat', q)"
    @convert-to-math="emit('convertToMath')"
  />
  <SpaceTiptapTableMenu :editor="editor" :writing="writing" />
  <SpaceTiptapTableEdgeControls :editor="editor" :prose-host="proseHost" />
  <SpaceTiptapLinkMenu :editor="editor" :writing="writing" />
  <SpaceTiptapDragHandle
    :editor="editor"
    :hover-host="articleHost"
    :prose-host="proseHost"
    :scroll-root="scrollRoot"
    :suppressed="writing.phase.value !== 'closed' || Boolean(mathTarget)"
    @ask-ai="(range) => writing.openPrompt(range)"
    @ask-in-chat="(q) => emit('askInChat', q)"
  />
  <SpaceTiptapMathPopover :editor="editor" :target="mathTarget" @close="emit('closeMath')" />
</template>
