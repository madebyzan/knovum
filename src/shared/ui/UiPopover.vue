<script setup lang="ts">
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import { computed, inject, provide, watch, type HTMLAttributes } from "vue";

import { overlayScopeVisibleKey } from "../contracts/overlay-scope-visibility";
import { cn } from "./cn";
import { POPUP_ANIMATE_CLASSES, POPOVER_SURFACE_CLASSES } from "./ui-classes";

const open = defineModel<boolean>("open", { default: false });
const scopeVisible = inject(overlayScopeVisibleKey, undefined);
const scopeActive = computed(() => scopeVisible?.value !== false);
provide(
  overlayScopeVisibleKey,
  computed(() => scopeActive.value && open.value),
);
watch(
  scopeActive,
  (visible) => {
    if (!visible) open.value = false;
  },
  { flush: "sync" },
);

const props = withDefaults(
  defineProps<{
    side?: "top" | "right" | "bottom" | "left" | undefined;
    align?: "start" | "center" | "end" | undefined;
    sideOffset?: number | undefined;
    alignOffset?: number | undefined;
    collisionPadding?: number | undefined;
    persistent?: boolean | undefined;
    forceMount?: boolean | undefined;
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
  <PopoverRoot v-if="scopeActive" v-model:open="open">
    <PopoverTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent
        :force-mount="forceMount"
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
            forceMount && 'data-[state=closed]:hidden',
            props.class,
          )
        "
      >
        <slot />
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
