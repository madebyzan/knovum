<script setup lang="ts">
import { computed, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";

import { cn } from "./cn";

const props = withDefaults(
  defineProps<{
    label?: string | undefined;
    size?: "sm" | "md" | undefined;
    tone?: "muted" | "accent" | "faint" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    label: undefined,
    size: "md",
    tone: "muted",
  },
);

const { t } = useI18n();
const accessibleLabel = computed(() => props.label ?? t("common.loading"));
const CLASSES_BY_TONE: Record<"muted" | "accent" | "faint", string> = {
  accent: "text-fg-accent",
  faint: "text-fg-tertiary",
  muted: "text-fg-secondary",
};
</script>

<template>
  <svg
    :class="
      cn(
        'shrink-0 animate-spin motion-reduce:animate-none',
        size === 'sm' ? 'size-3.5' : 'size-4',
        CLASSES_BY_TONE[tone],
        props.class,
      )
    "
    viewBox="0 0 24 24"
    fill="none"
    role="status"
    :aria-label="accessibleLabel"
    style="transform-origin: center; will-change: transform"
  >
    <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2.25" class="opacity-20" />
    <circle
      cx="12"
      cy="12"
      r="9"
      stroke="currentColor"
      stroke-width="2.25"
      stroke-linecap="round"
      stroke-dasharray="42 57"
    />
  </svg>
</template>
