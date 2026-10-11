<script setup lang="ts">
import { Eraser } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../selection/node-action-label";

withDefaults(defineProps<{ compact?: boolean; menuItem?: boolean; disabled?: boolean; reason?: string }>(), {
  compact: false,
  menuItem: false,
  disabled: false,
});
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
const REMOVE_OBJECT_LABEL_KEY = "spaces.media.imageEditor.removeObject.tool";
</script>

<template>
  <UiDropdownMenuItem
    v-if="menuItem"
    :disabled="disabled"
    :aria-label="disabled && reason ? `${t(REMOVE_OBJECT_LABEL_KEY)}：${reason}` : undefined"
    @select="emit('begin')"
  >
    <Eraser class="size-3.5" />{{ t(REMOVE_OBJECT_LABEL_KEY) }}
  </UiDropdownMenuItem>
  <UiTooltip
    v-else
    :text="disabled ? (reason ?? t(REMOVE_OBJECT_LABEL_KEY)) : t(REMOVE_OBJECT_LABEL_KEY)"
    :disabled="!compact && !disabled"
    side="top"
  >
    <ToolbarButton as-child>
      <button
        type="button"
        :class="CANVAS_ACTION_BUTTON_CLASSES"
        :aria-label="t(REMOVE_OBJECT_LABEL_KEY)"
        :disabled="disabled"
        @click="emit('begin')"
      >
        <Eraser class="size-4.5" :stroke-width="1.6" />
        <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{
          t(REMOVE_OBJECT_LABEL_KEY)
        }}</span>
      </button>
    </ToolbarButton>
  </UiTooltip>
</template>
