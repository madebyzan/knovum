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
  neutral: "bg-raised text-foreground-secondary",
  accent: "bg-accent-subtle text-accent",
  danger: "bg-danger-subtle text-danger",
  success: "bg-success-subtle text-success",
};
</script>

<template>
  <span
    :class="
      cn(
        'inline-flex shrink-0 items-center justify-center rounded-control font-medium tabular-nums select-none',
        size === 'sm' ? 'h-4 px-1.5 text-micro' : 'h-5 px-2 text-caption',
        variantClasses[variant],
        props.class,
      )
    "
  >
    <slot />
  </span>
</template>
