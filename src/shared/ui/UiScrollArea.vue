<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaViewport } from "reka-ui";
import { useTemplateRef } from "vue";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = defineProps<{
  class?: HTMLAttributes["class"];
  viewportClass?: HTMLAttributes["class"];
}>();
const emit = defineEmits<{ scroll: [] }>();
const viewport = useTemplateRef<{ viewportElement: HTMLElement }>("viewport");
defineExpose({
  get viewportElement() {
    return viewport.value?.viewportElement;
  },
});
</script>

<template>
  <ScrollAreaRoot :class="cn('relative overflow-hidden', props.class)">
    <ScrollAreaViewport
      ref="viewport"
      :class="cn('relative size-full overscroll-contain', props.viewportClass)"
      @scroll.passive="emit('scroll')"
      ><slot
    /></ScrollAreaViewport>
    <ScrollAreaScrollbar
      orientation="vertical"
      class="flex w-(--knovum-scrollbar-size) touch-none rounded-full bg-transparent p-(--knovum-scrollbar-inset) opacity-0 transition-[background-color,opacity] duration-150 select-none hover:bg-surface-control hover:opacity-100 data-[state=visible]:opacity-100"
    >
      <ScrollAreaThumb
        class="relative flex-1 rounded-full bg-(--knovum-scrollbar-thumb-bg) hover:bg-(--knovum-scrollbar-thumb-bg-hover)"
      />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
