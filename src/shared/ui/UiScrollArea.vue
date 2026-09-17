<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaViewport } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = defineProps<{
  class?: HTMLAttributes["class"];
}>();
</script>

<template>
  <ScrollAreaRoot :class="cn('relative overflow-hidden', props.class)">
    <ScrollAreaViewport class="size-full"><slot /></ScrollAreaViewport>
    <ScrollAreaScrollbar
      orientation="vertical"
      class="flex w-(--knovum-scrollbar-size) touch-none rounded-full bg-transparent p-(--knovum-scrollbar-inset) opacity-0 transition-[background-color,opacity] duration-150 select-none hover:bg-control hover:opacity-100 data-[state=visible]:opacity-100"
    >
      <ScrollAreaThumb class="bg-faint/55 hover:bg-muted/65 relative flex-1 rounded-full" />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
