<script setup lang="ts">
import { Download, Info } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceNodeTranscriptionAction from "../media/SpaceNodeTranscriptionAction.vue";
import SpaceNodeSpeechAction from "../speech/SpaceNodeSpeechAction.vue";

defineProps<{ disabled?: boolean; node?: SpaceCanvasNode | undefined }>();
const emit = defineEmits<{ inspect: []; export: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex h-full shrink-0 items-center gap-0.5 border-l border-stroke-divider px-1">
    <SpaceNodeSpeechAction :node="node" />
    <SpaceNodeTranscriptionAction
      v-if="node && (node.data.kind === 'audio' || node.data.kind === 'video')"
      :key="node.id"
      :node="node"
    />
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.inspector.title')" tooltip @click="emit('inspect')">
        <Info class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton as-child>
      <UiIconButton
        :label="t('spaces.canvas.nodeActions.export')"
        :disabled="disabled"
        tooltip
        @click="emit('export')"
      >
        <Download class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
  </div>
</template>
