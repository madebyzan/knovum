<script setup lang="ts">
import { StickyNote } from "@lucide/vue";

import type { PassageDrag } from "./use-passage-drag";

/** The passage under the pointer while it is dragged; clear only where it can be dropped. */
defineProps<{ drag: PassageDrag }>();
</script>

<template>
  <Teleport to="body">
    <div
      aria-hidden="true"
      class="pointer-events-none fixed z-80 flex max-w-64 items-center gap-1.5 rounded-md border border-stroke-default bg-surface-raised px-2 py-1 text-caption text-fg-primary shadow-popover transition-opacity duration-100"
      :class="drag.target ? 'opacity-100' : 'opacity-50'"
      :style="{ left: `${drag.x + 12}px`, top: `${drag.y + 12}px` }"
    >
      <StickyNote class="size-3.5 shrink-0" />
      <span class="truncate">{{ drag.passage.text }}</span>
    </div>
  </Teleport>
</template>
