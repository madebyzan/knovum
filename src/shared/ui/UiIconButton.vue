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
    /**
     * `primary` is the one committing action of a surface — sending a question, starting
     * playback. It is the solid neutral disc, never a brand colour: an accent fill reads as
     * decoration and competes with the content for attention.
     */
    tone?: "neutral" | "primary" | undefined;
    tooltipSide?: "top" | "right" | "bottom" | "left" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    active: false,
    disabled: false,
    size: "md",
    tone: "neutral",
    tooltipSide: "top",
  },
);
const attrs = useAttrs();
const TONES = {
  neutral:
    "rounded-md text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed data-[active=true]:bg-surface-control data-[active=true]:text-fg-primary",
  primary:
    "rounded-full bg-surface-inverse text-fg-inverse hover:bg-surface-inverse-hover hover:text-fg-inverse active:bg-surface-inverse-hover",
} as const;
</script>

<template>
  <UiTooltip
    :text="label"
    :shortcut="shortcut"
    :side="tooltipSide"
    :disabled="!(tooltip ?? Boolean(shortcut))"
  >
    <button
      v-bind="attrs"
      type="button"
      :aria-label="label"
      :aria-pressed="active || undefined"
      :disabled="disabled"
      :data-active="active"
      :class="
        cn(
          'grid shrink-0 place-items-center transition-[color,background-color,opacity] duration-100 ease-standard disabled:pointer-events-none disabled:opacity-40',
          TONES[tone],
          size === 'sm' ? 'size-(--knovum-control-height-sm)' : 'size-(--knovum-control-height-md)',
          props.class,
        )
      "
    >
      <slot />
    </button>
  </UiTooltip>
</template>
