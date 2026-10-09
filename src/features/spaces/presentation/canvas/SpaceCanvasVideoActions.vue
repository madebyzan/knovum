<script setup lang="ts">
import { Captions, Scissors } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SpaceCanvasActionTrigger from "./SpaceCanvasActionTrigger.vue";

defineProps<{ compact?: boolean; menuItem?: boolean; canAddSubtitles: boolean }>();
const emit = defineEmits<{ edit: []; addSubtitles: [] }>();
const { t } = useI18n();
const tools = [
  { id: "edit", icon: Scissors, label: "spaces.media.videoEditor.edit" },
  { id: "subtitles", icon: Captions, label: "spaces.creation.video.subtitles.add" },
] as const;
function invoke(actionId: "edit" | "subtitles") {
  if (actionId === "edit") emit("edit");
  else emit("addSubtitles");
}
</script>

<template>
  <SpaceCanvasActionTrigger
    v-for="tool in tools"
    :key="tool.id"
    :label="t(tool.label)"
    :icon="tool.icon"
    :compact="compact"
    :menu-item="menuItem"
    :disabled="tool.id === 'subtitles' && !canAddSubtitles"
    @invoke="invoke(tool.id)"
  />
</template>
