<script setup lang="ts">
import { RotateCcw, ScanText } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import SpaceCanvasActionTrigger from "../../canvas/presentation/SpaceCanvasActionTrigger.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useMediaKnowledge } from "../use-media-knowledge";

const props = defineProps<{ node: SpaceCanvasNode; compact?: boolean; menuItem?: boolean }>();
const { t } = useI18n();
const media = useMediaKnowledge(toRef(props, "node"));
const busy = computed(() => media.transcribing?.value ?? media.analyzing.value);
const label = computed(() =>
  t(
    `spaces.media.player.${busy.value ? "transcribing" : media.transcribed.value ? "retranscribe" : "transcribe"}`,
  ),
);
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="label"
    :icon="media.transcribed.value ? RotateCcw : ScanText"
    :disabled="!media.canTranscribe.value"
    :busy="busy"
    :compact="compact"
    :menu-item="menuItem"
    data-testid="node-transcription-action"
    @invoke="media.analyze"
  />
</template>
