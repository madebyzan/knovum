<script setup lang="ts">
import { Crop, RotateCw, SlidersHorizontal } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";

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
const TOOL_LABEL_KEYS: Record<ImageEditTool, string> = {
  adjust: "spaces.media.imageEditor.adjust.title",
  crop: "spaces.media.imageEditor.crop.title",
  transform: "spaces.media.transform.title",
};
const ICONS = { adjust: SlidersHorizontal, crop: Crop, transform: RotateCw };
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="t(TOOL_LABEL_KEYS[mode])"
    :icon="ICONS[mode]"
    :compact="compact"
    :menu-item="menuItem"
    :disabled="disabled"
    @invoke="emit('begin', props.mode)"
  />
</template>
