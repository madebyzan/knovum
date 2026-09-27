<script setup lang="ts">
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

type BadgeVariant = "neutral" | "accent" | "danger" | "success";
type BadgeSize = "sm" | "md";

const props = withDefaults(
  defineProps<{
    variant?: BadgeVariant | undefined;
    size?: BadgeSize | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    variant: "neutral",
    size: "md",
  },
);

const variantClasses: Record<BadgeVariant, string> = {
  neutral: "bg-surface-raised text-fg-secondary",
  accent: "bg-surface-accent-subtle text-fg-accent",
  danger: "bg-surface-danger-subtle text-fg-danger",
  success: "bg-surface-success-subtle text-fg-success",
};
</script>

<template>
  <span
    :class="
      cn(
        'inline-flex shrink-0 items-center justify-center rounded-md font-medium tabular-nums select-none',
        size === 'sm' ? 'h-4 px-1.5 text-caption-sm' : 'h-5 px-2 text-caption',
        variantClasses[variant],
        props.class,
      )
    "
  >
    <slot />
  </span>
</template>
