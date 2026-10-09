<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";
import SpaceCanvasActionTrigger from "../canvas/SpaceCanvasActionTrigger.vue";
import {
  nodeSpeechOperationIds,
  nodeSpeechOperations,
  type NodeSpeechOperation,
} from "./node-speech-operations";

const props = defineProps<{
  node: SpaceCanvasNode;
  compact?: boolean;
  menuItem?: boolean;
  separating?: boolean;
  separationBusy?: boolean;
}>();
const emit = defineEmits<{ begin: [operation: NodeSpeechOperation]; separate: [] }>();
const { t } = useI18n();
const store = useSpaceStore();
const { videos, backgrounds } = useSpeechMediaSources();
const canSeparate = computed(
  () =>
    Boolean(store.activeSpaceId) &&
    [...videos.value, ...backgrounds.value].some(
      ({ id, source }) => id === props.node.id && source.durationMs <= 600_000,
    ),
);
const operations = computed(() => nodeSpeechOperationIds(props.node));
</script>

<template>
  <SpaceCanvasActionTrigger
    v-for="operation in operations"
    :key="operation"
    :label="
      t(
        operation === 'separate' && separating
          ? 'spaces.speech.separation.cancel'
          : nodeSpeechOperations[operation].label,
      )
    "
    :icon="operation === 'separate' && separating ? LoaderCircle : nodeSpeechOperations[operation].icon"
    :compact="compact"
    :menu-item="menuItem"
    :disabled="operation === 'separate' && (!canSeparate || separationBusy)"
    :busy="operation === 'separate' && separating"
    data-testid="node-speech-action"
    :data-operation="operation"
    @invoke="operation === 'separate' ? emit('separate') : emit('begin', operation)"
  />
</template>
