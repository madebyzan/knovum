<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { NODE_SPEECH_OPERATIONS, type NodeSpeechOperation } from "./node-speech-operations";
import SpaceNodeSpeechEditor from "./SpaceNodeSpeechEditor.vue";
import SpaceVoiceCloneBar from "./SpaceVoiceCloneBar.vue";

const INSPECTOR_WIDTH_CLASSES: Record<Exclude<NodeSpeechOperation, "clone" | "separate">, string> = {
  short_dubbing: "w-[min(22.5rem,calc(100vw-2rem))]",
  audiobook: "w-[min(24.5rem,calc(100vw-2rem))]",
  video_dubbing: "w-[min(25rem,calc(100vw-2rem))]",
};

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
const panelStyle = computed(() => {
  const maxHeight = Math.min(props.availableHeight ?? 520, 520);
  return {
    maxWidth: props.availableWidth === undefined ? undefined : `${props.availableWidth}px`,
    maxHeight: `${maxHeight}px`,
    "--speech-scroll-max-h": `${Math.max(180, maxHeight - 124)}px`,
  };
});
watch([() => props.node?.id, () => props.node?.data.sourceRevision, () => store.activeSpaceId], () => {
  operation.value = undefined;
});
watch(
  operation,
  async (value) => {
    if (!value || value === "clone") return;
    await nextTick();
    const target = value === "separate" ? "button" : "section";
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
    :style="{ maxWidth: availableWidth === undefined ? undefined : `${availableWidth}px` }"
    data-testid="node-speech-controls"
    @keydown.esc.stop="close"
  >
    <SpaceVoiceCloneBar
      v-if="operation === 'clone'"
      :node="node"
      :available-width="availableWidth"
      @close="close"
    />
    <slot v-else-if="operation === 'separate'" name="separation" :close="close" />
    <section
      v-else
      tabindex="-1"
      :class="[
        'flex max-w-full flex-col overflow-hidden outline-none',
        POPOVER_SURFACE_CLASSES,
        INSPECTOR_WIDTH_CLASSES[operation],
      ]"
      :style="panelStyle"
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
