<script setup lang="ts">
import { RotateCcw, ScanText } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useLocalModelOptions } from "../../../../foundation/ai/use-model-choice-options";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import SpaceCanvasActionTrigger from "../../canvas/presentation/SpaceCanvasActionTrigger.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useMediaKnowledge } from "../use-media-knowledge";

const props = defineProps<{ node: SpaceCanvasNode; compact?: boolean; menuItem?: boolean }>();
const { t } = useI18n();
const media = useMediaKnowledge(toRef(props, "node"));
const { options: transcriptionOptions } = useLocalModelOptions(() => "transcription");
const busy = computed(() => media.transcribing?.value ?? media.analyzing.value);
const label = computed(() =>
  t(
    `spaces.media.player.${busy.value ? "transcribing" : media.transcribed.value ? "retranscribe" : "transcribe"}`,
  ),
);
</script>

<template>
  <ModelChoiceTrigger
    :options="transcriptionOptions"
    :menu-item="menuItem"
    :disabled="!media.canTranscribe.value"
    @select="(id) => media.analyze(id as LocalModelId)"
  >
    <template #menu-trigger>
      <component :is="media.transcribed.value ? RotateCcw : ScanText" class="size-3.5" />
      <span class="min-w-0 flex-1">{{ label }}</span>
    </template>
    <template #trigger="{ multiple }">
      <SpaceCanvasActionTrigger
        :label="label"
        :icon="media.transcribed.value ? RotateCcw : ScanText"
        :disabled="!media.canTranscribe.value"
        :busy="busy"
        :compact="compact"
        :menu-item="menuItem"
        data-testid="node-transcription-action"
        @invoke="!multiple && media.analyze()"
      />
    </template>
  </ModelChoiceTrigger>
</template>
