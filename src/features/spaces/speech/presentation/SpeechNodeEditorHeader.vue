<script setup lang="ts">
import { X } from "@lucide/vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../node-speech";
import { NODE_SPEECH_OPERATIONS } from "./node-speech-operations";

defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <header class="flex h-10 shrink-0 items-center gap-2 border-b border-stroke-divider px-3">
    <component
      :is="NODE_SPEECH_OPERATIONS[projectType].icon"
      class="size-3.5 shrink-0 text-fg-secondary"
      :stroke-width="1.7"
    />
    <div class="flex min-w-0 flex-1 items-baseline gap-1.5">
      <h2 class="shrink-0 text-label font-semibold text-fg-primary">
        {{ $t(NODE_SPEECH_OPERATIONS[projectType].label) }}
      </h2>
      <span
        v-if="node.data.title"
        class="min-w-0 truncate text-caption text-fg-tertiary"
        :title="node.data.title"
      >
        · {{ node.data.title }}
      </span>
    </div>
    <UiIconButton :label="$t('common.close')" size="sm" tooltip @click="emit('close')"
      ><X class="size-3.5"
    /></UiIconButton>
  </header>
</template>
