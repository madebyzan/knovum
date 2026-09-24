<script setup lang="ts">
import { Crop, Eraser, RotateCw, SlidersHorizontal } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import type { ImageEditTool } from "../../image-editing/image-edit-session";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

const props = withDefaults(
  defineProps<{
    mode: ImageEditTool;
    compact?: boolean;
    disabled?: boolean;
    menuItem?: boolean;
  }>(),
  { compact: false, disabled: false, menuItem: false },
);
const emit = defineEmits<{ begin: [tool: ImageEditTool] }>();
const { t } = useI18n();
const labels: Record<ImageEditTool, string> = {
  eraser: "spaces.media.eraser",
  adjust: "spaces.media.adjustImage",
  crop: "spaces.media.crop",
  transform: "spaces.media.transformImage",
};
const icons = { eraser: Eraser, adjust: SlidersHorizontal, crop: Crop, transform: RotateCw };
</script>

<template>
  <UiDropdownMenuItem v-if="menuItem" :disabled="disabled" @select="emit('begin', props.mode)">
    <component :is="icons[props.mode]" class="size-(--knovum-icon-size-sm)" />{{ t(labels[props.mode]) }}
  </UiDropdownMenuItem>
  <ToolbarButton v-else as-child>
    <button
      type="button"
      :class="CANVAS_ACTION_BUTTON_CLASSES"
      :aria-label="t(labels[props.mode])"
      :title="compact ? t(labels[props.mode]) : undefined"
      :disabled="disabled"
      @click="emit('begin', props.mode)"
    >
      <component :is="icons[props.mode]" class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
      <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{ t(labels[props.mode]) }}</span>
    </button>
  </ToolbarButton>
</template>
