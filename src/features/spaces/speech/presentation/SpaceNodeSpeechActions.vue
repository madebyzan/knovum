<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useLocalModelOptions } from "../../../../foundation/ai/use-model-choice-options";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import SpaceCanvasActionTrigger from "../../canvas/presentation/SpaceCanvasActionTrigger.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
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
const emit = defineEmits<{ begin: [operation: NodeSpeechOperation]; separate: [model?: LocalModelId] }>();
const { t } = useI18n();
const operations = computed(() => nodeSpeechOperationIds(props.node));
const { options: separationOptions } = useLocalModelOptions(() => "speechSeparation");
const separationDisabled = computed(
  () =>
    Boolean(props.separationUnavailableReason) || Boolean(props.separating) || Boolean(props.separationBusy),
);
</script>

<template>
  <template v-for="operation in operations" :key="operation">
    <ModelChoiceTrigger
      v-if="operation === 'separate'"
      :options="separationOptions"
      :menu-item="menuItem"
      :disabled="separationDisabled"
      @select="(id) => emit('separate', id as LocalModelId)"
    >
      <template #menu-trigger>
        <component :is="NODE_SPEECH_OPERATIONS.separate.icon" class="size-3.5" />
        <span class="min-w-0 flex-1">{{ t(NODE_SPEECH_OPERATIONS.separate.label) }}</span>
      </template>
      <template #trigger="{ multiple }">
        <SpaceCanvasActionTrigger
          :label="t(separating ? 'spaces.speech.separation.running' : NODE_SPEECH_OPERATIONS.separate.label)"
          :icon="separating ? LoaderCircle : NODE_SPEECH_OPERATIONS.separate.icon"
          :compact="compact"
          :menu-item="menuItem"
          :disabled="Boolean(separationUnavailableReason) || (separationBusy && !separating)"
          :unavailable-reason="separationUnavailableReason ? t(separationUnavailableReason) : undefined"
          :busy="separating"
          data-testid="node-speech-action"
          data-operation="separate"
          @invoke="!multiple && emit('separate')"
        />
      </template>
    </ModelChoiceTrigger>
    <SpaceCanvasActionTrigger
      v-else
      :label="t(NODE_SPEECH_OPERATIONS[operation].label)"
      :icon="NODE_SPEECH_OPERATIONS[operation].icon"
      :compact="compact"
      :menu-item="menuItem"
      data-testid="node-speech-action"
      :data-operation="operation"
      @invoke="emit('begin', operation)"
    />
  </template>
</template>
