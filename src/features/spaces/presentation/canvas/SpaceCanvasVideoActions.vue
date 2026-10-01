<script setup lang="ts">
import { Captions, Scissors } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";

defineProps<{ compact?: boolean; menuItem?: boolean; canAddSubtitles: boolean }>();
const emit = defineEmits<{ edit: []; addSubtitles: [] }>();
const { t } = useI18n();
const tools = [
  { id: "edit", icon: Scissors, label: "spaces.media.videoEditor.edit" },
  { id: "subtitles", icon: Captions, label: "spaces.creation.video.subtitles.add" },
] as const;
function invoke(id: "edit" | "subtitles") {
  if (id === "edit") emit("edit");
  else emit("addSubtitles");
}
</script>

<template>
  <template v-for="tool in tools" :key="tool.id">
    <UiDropdownMenuItem
      v-if="menuItem"
      :disabled="tool.id === 'subtitles' && !canAddSubtitles"
      @select="invoke(tool.id)"
    >
      <component :is="tool.icon" class="size-3.5" />{{ t(tool.label) }}
    </UiDropdownMenuItem>
    <UiTooltip v-else :text="t(tool.label)" :disabled="!compact" side="top">
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t(tool.label)"
          :disabled="tool.id === 'subtitles' && !canAddSubtitles"
          @click="invoke(tool.id)"
        >
          <component :is="tool.icon" class="size-3.5" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{ t(tool.label) }}</span>
        </button>
      </ToolbarButton>
    </UiTooltip>
  </template>
</template>
