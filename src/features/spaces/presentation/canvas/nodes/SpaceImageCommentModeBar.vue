<script setup lang="ts">
import { MessageCircle, Pencil, ScanLine, X } from "@lucide/vue";

import type { ImageCommentMode } from "../../../comments/image-comment-session";

defineProps<{ mode: ImageCommentMode }>();
const emit = defineEmits<{ change: [mode: ImageCommentMode]; cancel: [] }>();
const modes = [
  { id: "point", icon: MessageCircle, label: "spaces.media.imageEditor.comments.point" },
  { id: "draw", icon: Pencil, label: "spaces.media.imageEditor.comments.draw" },
  { id: "region", icon: ScanLine, label: "spaces.media.imageEditor.comments.region" },
] as const;
</script>

<template>
  <div
    class="nodrag pointer-events-auto absolute bottom-3 left-1/2 z-30 flex -translate-x-1/2 items-center gap-1 rounded-panel border border-border bg-overlay p-1 shadow-popover"
    role="toolbar"
    :aria-label="$t('spaces.media.imageEditor.comments.mode')"
  >
    <button
      v-for="item in modes"
      :key="item.id"
      type="button"
      class="flex size-9 items-center justify-center rounded-control text-foreground-secondary transition-colors outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :class="mode === item.id && 'bg-control text-foreground'"
      :aria-label="$t(item.label)"
      :aria-pressed="mode === item.id"
      @click.stop="emit('change', item.id)"
    >
      <component :is="item.icon" class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.7" />
    </button>
    <span aria-hidden="true" class="mx-0.5 h-4 w-px bg-border" />
    <button
      type="button"
      class="flex size-9 items-center justify-center rounded-control text-foreground-secondary transition-colors outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :aria-label="$t('common.cancel')"
      @click.stop="emit('cancel')"
    >
      <X class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.7" />
    </button>
  </div>
</template>
