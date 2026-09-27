<script setup lang="ts">
import type { HTMLAttributes } from "vue";
import { useAttrs } from "vue";

import { cn } from "./cn";
import UiTooltip from "./UiTooltip.vue";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    label: string;
    shortcut?: string | undefined;
    active?: boolean | undefined;
    disabled?: boolean | undefined;
    tooltip?: boolean | undefined;
    size?: "sm" | "md" | undefined;
    tooltipSide?: "top" | "right" | "bottom" | "left" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    active: false,
    disabled: false,
    tooltip: true,
    size: "md",
    tooltipSide: "top",
  },
);
const attrs = useAttrs();
</script>

<template>
  <UiTooltip :text="label" :shortcut="shortcut" :side="tooltipSide" :disabled="!tooltip">
    <button
      v-bind="attrs"
      type="button"
      :aria-label="label"
      :aria-pressed="active || undefined"
      :disabled="disabled"
      :data-active="active"
      :class="
        cn(
          'grid shrink-0 place-items-center rounded-md text-fg-secondary transition-[color,background-color] duration-100 ease-standard hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed disabled:pointer-events-none disabled:opacity-35 data-[active=true]:bg-surface-control data-[active=true]:text-fg-primary',
          size === 'sm' ? 'size-(--knovum-control-height-sm)' : 'size-(--knovum-control-height-md)',
          props.class,
        )
      "
    >
      <slot />
    </button>
  </UiTooltip>
</template>
