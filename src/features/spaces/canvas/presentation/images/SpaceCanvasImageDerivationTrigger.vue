<script setup lang="ts">
import { ToolbarButton } from "reka-ui";
import type { Component } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, shouldShowAction } from "../../../../../shared/contracts/action-availability";
import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../selection/node-action-label";
import type { ImageActionAvailability } from "./image-action-availability";

/** One image derivation offered directly on the node bar. */
defineProps<{
  compact: boolean;
  availability: ImageActionAvailability;
  reason: string;
  icon: Component;
  labelKey: string;
}>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
</script>

<template>
  <UiTooltip
    v-if="shouldShowAction(availability)"
    :text="canRunAction(availability) ? t(labelKey) : reason"
    :disabled="canRunAction(availability) && !compact"
    side="top"
  >
    <ToolbarButton as-child>
      <button
        type="button"
        :class="CANVAS_ACTION_BUTTON_CLASSES"
        :aria-disabled="!canRunAction(availability)"
        :aria-label="t(labelKey)"
        @click="emit('begin')"
      >
        <component :is="icon" class="size-4.5" :stroke-width="1.6" />
        <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{ t(labelKey) }}</span>
      </button>
    </ToolbarButton>
  </UiTooltip>
</template>
