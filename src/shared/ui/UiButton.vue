<script setup lang="ts">
import { Primitive } from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "dangerSubtle";
type ButtonSize = "sm" | "md";

const props = withDefaults(
  defineProps<{
    variant?: ButtonVariant | undefined;
    size?: ButtonSize | undefined;
    disabled?: boolean | undefined;
    asChild?: boolean | undefined;
    as?: string | object | undefined;
    type?: "button" | "submit" | "reset" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    variant: "secondary",
    size: "md",
    disabled: false,
    asChild: false,
    as: "button",
    type: "button",
  },
);

const variantClass = computed(
  () =>
    ({
      primary:
        "bg-surface-accent text-fg-on-accent hover:bg-surface-accent-hover active:bg-surface-accent-pressed",
      secondary:
        "bg-surface-control text-fg-primary hover:bg-surface-control-hover active:bg-surface-control-pressed",
      ghost:
        "bg-transparent text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed shadow-none",
      danger: "bg-surface-danger text-fg-on-danger hover:opacity-90 active:opacity-85",
      dangerSubtle:
        "bg-surface-danger-subtle text-fg-danger shadow-none hover:bg-surface-danger-subtle-hover hover:text-fg-danger active:bg-surface-danger-subtle-pressed",
    })[props.variant],
);
</script>

<template>
  <Primitive
    :as="as"
    :as-child="asChild"
    :type="as === 'button' ? type : undefined"
    :disabled="disabled"
    :class="
      cn(
        'inline-flex shrink-0 items-center justify-center gap-1.5 rounded-md px-2.5 text-label font-medium shadow-control transition-[color,background-color,box-shadow] duration-100 ease-standard disabled:pointer-events-none disabled:opacity-40',
        variantClass,
        size === 'sm' ? 'h-(--knovum-control-height-sm)' : 'h-(--knovum-control-height-md)',
        props.class,
      )
    "
  >
    <slot />
  </Primitive>
</template>
