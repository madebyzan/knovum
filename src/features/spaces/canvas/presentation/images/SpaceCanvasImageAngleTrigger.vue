<script setup lang="ts">
import { Box } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../selection/node-action-label";

defineProps<{ compact: boolean }>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
const ANGLE_LABEL_KEY = "spaces.media.imageEditor.angle.title";
</script>

<template>
  <ToolbarButton as-child>
    <button
      type="button"
      :class="CANVAS_ACTION_BUTTON_CLASSES"
      :aria-label="t(ANGLE_LABEL_KEY)"
      :title="compact ? t(ANGLE_LABEL_KEY) : undefined"
      @click="emit('begin')"
    >
      <Box class="size-4.5" :stroke-width="1.6" />
      <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{ t(ANGLE_LABEL_KEY) }}</span>
    </button>
  </ToolbarButton>
</template>
