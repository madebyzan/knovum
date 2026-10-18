<script setup lang="ts">
import { ToolbarButton } from "reka-ui";
import type { Component } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, shouldShowAction } from "../../../../../shared/contracts/action-availability";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../selection/node-action-label";
import type { ImageActionAvailability } from "./image-action-availability";

/**
 * One image action offered on the node bar, or in More once the bar runs out of room. An action the
 * node cannot run yet stays visible and says why, so the bar does not change shape while the image
 * is still being prepared.
 */
defineProps<{
  availability: ImageActionAvailability;
  reason: string;
  icon: Component;
  labelKey: string;
  compact?: boolean;
  menuItem?: boolean;
}>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
</script>

<template>
  <template v-if="shouldShowAction(availability)">
    <UiDropdownMenuItem
      v-if="menuItem"
      :disabled="!canRunAction(availability)"
      :aria-label="canRunAction(availability) ? undefined : `${t(labelKey)}：${reason}`"
      @select="emit('begin')"
    >
      <component :is="icon" class="size-3.5" />{{ t(labelKey) }}
    </UiDropdownMenuItem>
    <UiTooltip
      v-else
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
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
            t(labelKey)
          }}</span>
        </button>
      </ToolbarButton>
    </UiTooltip>
  </template>
</template>
