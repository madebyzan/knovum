<script setup lang="ts">
import { MessageCircle } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type { ImageEditTool } from "../../image-editing/image-edit-session-store";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { IMAGE_EDIT_TOOL_ORDER } from "./canvas-bar-density-policy";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";
import SpaceCanvasImageToolTrigger from "./SpaceCanvasImageToolTrigger.vue";

defineProps<{ compact: boolean; dimensionsReady: boolean }>();
const emit = defineEmits<{ beginTool: [tool: ImageEditTool]; beginComment: [] }>();
const { t } = useI18n();
</script>

<template>
  <UiTooltip :text="t('spaces.media.imageComments.title')" :disabled="!compact" side="top">
    <ToolbarButton as-child>
      <button
        type="button"
        :class="CANVAS_ACTION_BUTTON_CLASSES"
        :aria-label="t('spaces.media.imageComments.title')"
        @click="emit('beginComment')"
      >
        <MessageCircle class="size-4.5" :stroke-width="1.6" />
        <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{
          t("spaces.media.imageComments.title")
        }}</span>
      </button>
    </ToolbarButton>
  </UiTooltip>
  <SpaceCanvasImageToolTrigger
    v-for="mode in IMAGE_EDIT_TOOL_ORDER"
    :key="mode"
    :mode="mode"
    :compact="compact"
    :disabled="mode === 'crop' && !dimensionsReady"
    @begin="emit('beginTool', $event)"
  />
  <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
</template>
