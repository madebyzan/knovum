<script setup lang="ts">
import { Box } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

defineProps<{ compact: boolean }>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
const label = "spaces.media.imageEditor.angle.title";
</script>

<template>
  <ToolbarButton as-child>
    <button
      type="button"
      :class="CANVAS_ACTION_BUTTON_CLASSES"
      :aria-label="t(label)"
      :title="compact ? t(label) : undefined"
      @click="emit('begin')"
    >
      <Box class="size-4.5" :stroke-width="1.6" />
      <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{ t(label) }}</span>
    </button>
  </ToolbarButton>
</template>
