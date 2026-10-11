<script setup lang="ts">
import { ArrowLeft, X } from "@lucide/vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { NODE_SPEECH_OPERATIONS } from "./node-speech-operations";

defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType; isAddingVoice: boolean }>();
const emit = defineEmits<{ back: []; close: [] }>();
</script>

<template>
  <header class="flex shrink-0 items-center gap-2 border-b border-stroke-divider px-3 py-2">
    <UiIconButton v-if="isAddingVoice" size="sm" :label="$t('common.back')" tooltip @click="emit('back')">
      <ArrowLeft class="size-3.5" />
    </UiIconButton>
    <component
      v-else
      :is="NODE_SPEECH_OPERATIONS[projectType].icon"
      class="size-4 shrink-0 text-fg-secondary"
      :stroke-width="1.6"
    />
    <h2 class="min-w-0 flex-1 text-caption font-medium" :title="node.data.title">
      {{ $t(isAddingVoice ? "spaces.speech.library.add" : NODE_SPEECH_OPERATIONS[projectType].label) }}
    </h2>
    <UiIconButton :label="$t('common.close')" size="sm" tooltip @click="emit('close')"
      ><X class="size-3.5"
    /></UiIconButton>
  </header>
</template>
