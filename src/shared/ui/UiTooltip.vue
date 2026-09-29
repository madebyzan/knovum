<script setup lang="ts">
import {
  TooltipArrow,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger,
} from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";
import { POPUP_ANIMATE_CLASSES } from "./ui-classes";
import UiKbd from "./UiKbd.vue";

const props = withDefaults(
  defineProps<{
    text: string;
    disabled?: boolean | undefined;
    shortcut?: string | undefined;
    side?: "top" | "right" | "bottom" | "left" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    disabled: false,
    side: "right",
  },
);
</script>

<template>
  <slot v-if="disabled" />
  <TooltipProvider v-else :delay-duration="300" :skip-delay-duration="100">
    <TooltipRoot :ignore-non-keyboard-focus="true">
      <TooltipTrigger as-child><slot /></TooltipTrigger>
      <TooltipPortal>
        <TooltipContent
          :side="side"
          :side-offset="7"
          :collision-padding="8"
          avoid-collisions
          :class="
            cn(
              POPUP_ANIMATE_CLASSES,
              'pointer-events-none z-70 rounded-md border border-stroke-default material-overlay px-2 py-1 text-caption text-fg-primary shadow-popover duration-100 ease-standard select-none',
              props.class,
            )
          "
        >
          <span class="flex items-center gap-2">
            <span>{{ text }}</span>
            <UiKbd v-if="shortcut" :shortcut="shortcut" variant="badge" class="-mr-0.5" />
          </span>
          <TooltipArrow class="fill-background-overlay" />
        </TooltipContent>
      </TooltipPortal>
    </TooltipRoot>
  </TooltipProvider>
</template>
