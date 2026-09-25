<script setup lang="ts">
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";
import { POPUP_ANIMATE_CLASSES, POPOVER_SURFACE_CLASSES } from "./ui-classes";
import UiTooltip from "./UiTooltip.vue";

const open = defineModel<boolean>("open", { default: false });

const props = withDefaults(
  defineProps<{
    side?: "top" | "right" | "bottom" | "left" | undefined;
    align?: "start" | "center" | "end" | undefined;
    sideOffset?: number | undefined;
    alignOffset?: number | undefined;
    collisionPadding?: number | undefined;
    persistent?: boolean | undefined;
    triggerTooltip?: string | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    side: "bottom",
    align: "center",
    sideOffset: 6,
    alignOffset: 0,
    collisionPadding: 8,
  },
);

function handleOutsideInteraction(event: Event) {
  if (props.persistent) event.preventDefault();
}
</script>

<template>
  <PopoverRoot v-model:open="open">
    <UiTooltip v-if="$slots.trigger && triggerTooltip" :text="triggerTooltip" side="top">
      <PopoverTrigger as-child><slot name="trigger" /></PopoverTrigger>
    </UiTooltip>
    <PopoverTrigger v-else-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent
        :side="side"
        :align="align"
        :side-offset="sideOffset"
        :align-offset="alignOffset"
        :collision-padding="collisionPadding"
        avoid-collisions
        @interact-outside="handleOutsideInteraction"
        :class="
          cn(
            POPUP_ANIMATE_CLASSES,
            POPOVER_SURFACE_CLASSES,
            'z-50 p-2 duration-100 ease-standard outline-none select-none',
            props.class,
          )
        "
      >
        <slot />
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
