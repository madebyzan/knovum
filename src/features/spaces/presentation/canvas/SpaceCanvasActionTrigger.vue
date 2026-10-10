<script setup lang="ts">
import { ToolbarButton } from "reka-ui";
import type { Component } from "vue";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

defineOptions({ inheritAttrs: false });
defineProps<{
  label: string;
  icon: Component;
  compact?: boolean;
  menuItem?: boolean;
  disabled?: boolean;
  busy?: boolean;
  unavailableReason?: string | undefined;
}>();
const emit = defineEmits<{ invoke: [] }>();
</script>

<template>
  <UiDropdownMenuItem
    v-if="menuItem"
    v-bind="$attrs"
    :disabled="disabled"
    :aria-label="label"
    :title="unavailableReason"
    @select="emit('invoke')"
  >
    <component :is="icon" class="size-3.5" :class="{ 'animate-spin motion-reduce:animate-none': busy }" />{{
      label
    }}
  </UiDropdownMenuItem>
  <UiTooltip v-else :text="unavailableReason || label" :disabled="!compact && !unavailableReason" side="top">
    <ToolbarButton as-child>
      <button
        v-bind="$attrs"
        type="button"
        :class="CANVAS_ACTION_BUTTON_CLASSES"
        :aria-label="label"
        :title="unavailableReason"
        :disabled="disabled"
        @click="emit('invoke')"
      >
        <component
          :is="icon"
          class="size-4.5"
          :class="{ 'animate-spin motion-reduce:animate-none': busy }"
          :stroke-width="1.6"
        />
        <span :class="[nodeActionLabelClass, nodeActionLabelState(Boolean(compact))]">{{ label }}</span>
      </button>
    </ToolbarButton>
  </UiTooltip>
</template>
