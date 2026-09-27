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
</script>

<template>
  <SplitterResizeHandle
    :class="cn('group relative z-30 w-0 shrink-0 self-stretch bg-transparent outline-none', props.class)"
    :disabled="disabled"
    :tabindex="disabled ? -1 : 0"
  >
    <span v-show="!disabled" class="absolute inset-y-0 -left-2 w-4">
      <span
        class="group-hover:bg-border-default absolute inset-y-2 left-2 w-px rounded-full bg-transparent transition-colors duration-100 group-focus-visible:bg-surface-accent group-data-[state=drag]:bg-surface-accent"
      />
    </span>
  </SplitterResizeHandle>
</template>
