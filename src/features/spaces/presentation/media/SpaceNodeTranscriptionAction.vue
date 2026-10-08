<script setup lang="ts">
import { LoaderCircle, ScanText } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useMediaKnowledge } from "../../media/use-media-knowledge";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const { canTranscribe, transcribed, analyzing, analyze } = useMediaKnowledge(toRef(props, "node"));
const label = computed(() =>
  t(
    `spaces.media.player.${transcribed.value ? "transcribed" : analyzing.value ? "transcribing" : "transcribe"}`,
  ),
);
</script>

<template>
  <ToolbarButton as-child>
    <UiIconButton
      :label="label"
      :disabled="!canTranscribe"
      tooltip
      data-testid="node-transcription-action"
      @click="analyze"
    >
      <LoaderCircle v-if="analyzing" class="size-3.5 animate-spin" />
      <ScanText v-else class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
  </ToolbarButton>
</template>
