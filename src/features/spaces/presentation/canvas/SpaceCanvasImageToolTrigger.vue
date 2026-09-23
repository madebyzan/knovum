<script setup lang="ts">
import { Crop, Eraser, RotateCw, SlidersHorizontal } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import type { ImageEditTool } from "../../image-editing/image-edit-session";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

const props = defineProps<{ mode: ImageEditTool; compact: boolean; disabled?: boolean }>();
const emit = defineEmits<{ begin: [tool: ImageEditTool] }>();
const { t } = useI18n();
const labels: Record<ImageEditTool, string> = {
  eraser: "spaces.media.eraser",
  adjust: "spaces.media.adjustImage",
  crop: "spaces.media.crop",
  transform: "spaces.media.transformImage",
};
const icons = { eraser: Eraser, adjust: SlidersHorizontal, crop: Crop, transform: RotateCw };
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-150 ease-standard hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-45 motion-reduce:transition-none";
</script>

<template>
  <ToolbarButton as-child>
    <button
      type="button"
      :class="commandClass"
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
