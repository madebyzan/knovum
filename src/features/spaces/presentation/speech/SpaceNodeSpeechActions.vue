<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceCanvasActionTrigger from "../canvas/SpaceCanvasActionTrigger.vue";
import {
  nodeSpeechOperationIds,
  NODE_SPEECH_OPERATIONS,
  type NodeSpeechOperation,
} from "./node-speech-operations";

const props = defineProps<{
  node: SpaceCanvasNode;
  compact?: boolean;
  menuItem?: boolean;
  separating?: boolean;
  separationBusy?: boolean;
  separationUnavailableReason: string | undefined;
}>();
const emit = defineEmits<{ begin: [operation: NodeSpeechOperation]; separate: [] }>();
const { t } = useI18n();
const operations = computed(() => nodeSpeechOperationIds(props.node));
</script>

<template>
  <SpaceCanvasActionTrigger
    v-for="operation in operations"
    :key="operation"
    :label="
      t(
        operation === 'separate' && separating
          ? 'spaces.speech.separation.running'
          : NODE_SPEECH_OPERATIONS[operation].label,
      )
    "
    :icon="operation === 'separate' && separating ? LoaderCircle : NODE_SPEECH_OPERATIONS[operation].icon"
    :compact="compact"
    :menu-item="menuItem"
    :disabled="
      operation === 'separate' && (Boolean(separationUnavailableReason) || (separationBusy && !separating))
    "
    :unavailable-reason="
      operation === 'separate' && separationUnavailableReason ? t(separationUnavailableReason) : undefined
    "
    :busy="operation === 'separate' && separating"
    data-testid="node-speech-action"
    :data-operation="operation"
    @invoke="operation === 'separate' ? emit('separate') : emit('begin', operation)"
  />
</template>
