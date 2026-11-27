<script setup lang="ts">
import {
  injectTooltipProviderContext,
  TooltipArrow,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger,
} from "reka-ui";
import { shallowRef, watch, type FunctionalComponent, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { OVERLAY_LAYER_CLASSES, POPUP_ANIMATE_CLASSES } from "./ui-classes";
import UiKbd from "./UiKbd.vue";

const Passthrough: FunctionalComponent = (_, { slots }) => slots.default?.();

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

const open = shallowRef(false);
// Inherit the application-wide TooltipProvider so sibling tooltips share skip-delay-duration;
// fall back to a local provider only in isolated component mounts (e.g. unit tests).
const hasProvider = injectTooltipProviderContext(null) !== null;
const ProviderHost = hasProvider ? Passthrough : TooltipProvider;
const providerProps = hasProvider ? {} : { delayDuration: 300, skipDelayDuration: 100 };

// The root keeps an open tooltip mounted when it becomes disabled, and the trigger drops its
// listeners at the same time, so nothing else would close it.
watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) open.value = false;
  },
);
</script>

<template>
  <component :is="ProviderHost" v-bind="providerProps">
    <TooltipRoot v-model:open="open" :ignore-non-keyboard-focus="true" :disabled="disabled">
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
              OVERLAY_LAYER_CLASSES.tooltip,
              'pointer-events-none rounded-md border border-stroke-default material-overlay px-2 py-1 text-caption text-fg-primary shadow-popover duration-100 ease-standard select-none',
              props.class,
            )
          "
        >
          <span class="flex items-center gap-2">
            <span>{{ text }}</span>
            <UiKbd v-if="shortcut" :shortcut="shortcut" variant="badge" class="-mr-0.5" />
          </span>
          <TooltipArrow class="fill-surface-overlay" />
        </TooltipContent>
      </TooltipPortal>
    </TooltipRoot>
  </component>
</template>
