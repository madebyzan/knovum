<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "../../../../shared/ui/cn";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";

/** An action-bar control that opens a menu: its icon or current value, then a chevron. */
defineProps<{ label: string; disabled?: boolean | undefined; class?: HTMLAttributes["class"] }>();
</script>

<template>
  <ToolbarButton as-child>
    <button
      type="button"
      :class="cn(CANVAS_ACTION_BUTTON_CLASSES, 'gap-1 px-2', $props.class)"
      :aria-label="label"
      :title="label"
      :disabled="disabled"
    >
      <slot />
      <ChevronDown class="size-3 shrink-0 text-fg-tertiary" />
    </button>
  </ToolbarButton>
</template>
