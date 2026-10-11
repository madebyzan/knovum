<script setup lang="ts">
import { LoaderCircle, ScanText } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import SpaceCanvasActionTrigger from "../../canvas/presentation/SpaceCanvasActionTrigger.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useMediaKnowledge } from "../use-media-knowledge";

const props = defineProps<{ node: SpaceCanvasNode; compact?: boolean; menuItem?: boolean }>();
const { t } = useI18n();
const { canTranscribe, transcribed, analyzing, analyze } = useMediaKnowledge(toRef(props, "node"));
const label = computed(() =>
  t(
    `spaces.media.player.${transcribed.value ? "transcribed" : analyzing.value ? "transcribing" : "transcribe"}`,
  ),
);
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="label"
    :icon="analyzing ? LoaderCircle : ScanText"
    :disabled="!canTranscribe"
    :busy="analyzing"
    :compact="compact"
    :menu-item="menuItem"
    data-testid="node-transcription-action"
    @invoke="analyze"
  />
</template>
