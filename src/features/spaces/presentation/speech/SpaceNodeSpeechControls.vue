<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { NODE_SPEECH_OPERATIONS, type NodeSpeechOperation } from "./node-speech-operations";
import SpaceNodeSpeechEditor from "./SpaceNodeSpeechEditor.vue";
import SpaceVoiceCloneBar from "./SpaceVoiceCloneBar.vue";

const props = defineProps<{
  node?: SpaceCanvasNode | undefined;
  availableWidth?: number;
  availableHeight?: number | undefined;
}>();
const operation = defineModel<NodeSpeechOperation | undefined>("operation", { required: true });
const { t } = useI18n();
const store = useSpaceStore();
const controlsRef = useTemplateRef<HTMLElement>("controlsRef");
const projectEditorRef = useTemplateRef("projectEditorRef");
async function close() {
  if (projectEditorRef.value && !(await projectEditorRef.value.requestClose())) return;
  operation.value = undefined;
}
watch([() => props.node?.id, () => props.node?.data.sourceRevision, () => store.activeSpaceId], () => {
  operation.value = undefined;
});
watch(
  operation,
  async (value) => {
    if (!value) return;
    await nextTick();
    const target = value === "clone" ? "button" : "section";
    controlsRef.value?.querySelector<HTMLElement>(target)?.focus({ preventScroll: true });
  },
  { immediate: true, flush: "post" },
);
</script>

<template>
  <div
    v-if="node && operation"
    ref="controlsRef"
    class="w-max max-w-full"
    data-testid="node-speech-controls"
    @keydown.esc.stop="close"
  >
    <SpaceVoiceCloneBar
      v-if="operation === 'clone'"
      :node="node"
      :compact="availableWidth !== undefined && availableWidth < 520"
      @close="close"
    />
    <section
      v-else
      tabindex="-1"
      class="flex max-h-[calc(100vh-8rem)] w-[min(24rem,calc(100vw-2rem))] max-w-full flex-col overflow-hidden rounded-panel border border-stroke-strong bg-surface-overlay shadow-popover"
      :style="{
        maxWidth: availableWidth === undefined ? undefined : `${availableWidth}px`,
        maxHeight: availableHeight === undefined ? undefined : `${availableHeight}px`,
      }"
      :aria-label="t(NODE_SPEECH_OPERATIONS[operation].label)"
      :data-source-node-id="node.id"
    >
      <SpaceNodeSpeechEditor
        ref="projectEditorRef"
        :key="`${node.id}:${operation}`"
        :node="node"
        :project-type="operation"
        @close="close"
      />
    </section>
  </div>
</template>
