<script setup lang="ts">
import { Separator } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = withDefaults(
  defineProps<{
    orientation?: "horizontal" | "vertical" | undefined;
    decorative?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    orientation: "horizontal",
    decorative: true,
  },
);
</script>

<template>
  <Separator
    :orientation="orientation"
    :decorative="decorative"
    :class="
      cn(
        'bg-border-divider shrink-0',
        orientation === 'horizontal' ? 'h-px w-full' : 'h-3.5 w-px',
        props.class,
      )
    "
  />
</template>
