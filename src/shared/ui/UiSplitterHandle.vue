<script setup lang="ts">
import { SplitterResizeHandle } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = withDefaults(
  defineProps<{
    disabled?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
  },
);
const emit = defineEmits<{ dragging: [value: boolean] }>();
</script>

<template>
  <SplitterResizeHandle
    :class="cn('group relative z-30 w-0 shrink-0 self-stretch bg-transparent outline-none', props.class)"
    :disabled="disabled"
    :tabindex="disabled ? -1 : 0"
    @dragging="emit('dragging', $event)"
  >
    <span v-show="!disabled" class="absolute inset-y-0 -left-2.5 flex w-5 cursor-col-resize justify-center">
      <span
        class="absolute inset-y-2 w-0.5 rounded-full bg-transparent transition-colors duration-100 group-hover:bg-surface-accent group-focus-visible:bg-surface-accent group-data-[state=drag]:bg-surface-accent"
      />
    </span>
  </SplitterResizeHandle>
</template>
