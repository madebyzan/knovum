<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
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
const TONE_CLASSES: Record<"muted" | "accent" | "faint", string> = {
  accent: "text-fg-accent",
  faint: "text-fg-tertiary",
  muted: "text-fg-secondary",
};
</script>

<template>
  <LoaderCircle
    :class="
      cn(
        'animate-spin motion-reduce:animate-none',
        size === 'sm' ? 'size-3.5' : 'size-4',
        TONE_CLASSES[tone],
        props.class,
      )
    "
    :stroke-width="1.75"
    role="status"
    :aria-label="accessibleLabel"
  />
</template>
