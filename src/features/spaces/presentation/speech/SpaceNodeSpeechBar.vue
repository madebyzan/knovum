<script setup lang="ts">
import { Pencil, X } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import { nodeSpeechOperations, type NodeSpeechOperation } from "./node-speech-operations";
import SpaceNodeSpeechEditor from "./SpaceNodeSpeechEditor.vue";
import SpaceVoiceCloneBar from "./SpaceVoiceCloneBar.vue";

const props = defineProps<{ node?: SpaceCanvasNode | undefined; availableWidth?: number }>();
const operation = defineModel<NodeSpeechOperation | undefined>("operation", { required: true });
const { t } = useI18n();
const store = useSpaceStore();
const bar = useTemplateRef<HTMLElement>("bar");
const projectEditor = useTemplateRef("projectEditor");
const projectOpen = shallowRef(true);
async function close() {
  if (projectEditor.value && !(await projectEditor.value.requestClose())) return;
  operation.value = undefined;
}
watch([() => props.node?.id, () => props.node?.data.sourceRevision, () => store.activeSpaceId], () => {
  operation.value = undefined;
});
watch(
  operation,
  async (value) => {
    if (!value) return;
    projectOpen.value = true;
    if (value !== "clone") return;
    await nextTick();
    bar.value?.querySelector<HTMLButtonElement>("button")?.focus({ preventScroll: true });
  },
  { immediate: true, flush: "post" },
);
</script>

<template>
  <div
    v-if="node && operation"
    ref="bar"
    class="w-max max-w-full"
    data-testid="node-speech-bar"
    @keydown.esc.stop="close"
  >
    <SpaceVoiceCloneBar
      v-if="operation === 'clone'"
      :node="node"
      :compact="availableWidth !== undefined && availableWidth < 520"
      @close="close"
    />
    <ToolbarRoot
      v-else
      :class="CANVAS_ACTION_BAR_CLASSES"
      :aria-label="t(nodeSpeechOperations[operation].label)"
      :loop="false"
    >
      <component
        :is="nodeSpeechOperations[operation].icon"
        class="ml-2 size-4 text-fg-secondary"
        :stroke-width="1.6"
      />
      <span class="px-1.5 text-label font-medium">{{ t(nodeSpeechOperations[operation].label) }}</span>
      <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
      <UiPopover
        v-model:open="projectOpen"
        force-mount
        align="start"
        class="w-[min(24rem,calc(100vw-2rem))] p-3"
      >
        <template #trigger
          ><ToolbarButton as-child
            ><button type="button" :class="CANVAS_ACTION_BUTTON_CLASSES">
              <Pencil class="size-4" :stroke-width="1.6" />{{ t("spaces.speech.project.edit") }}
            </button></ToolbarButton
          ></template
        >
        <UiScrollArea
          class="max-h-[min(36rem,calc(100vh-6rem))]"
          viewport-class="max-h-[min(36rem,calc(100vh-6rem))]"
        >
          <SpaceNodeSpeechEditor
            ref="projectEditor"
            :key="`${node.id}:${operation}`"
            :node="node"
            :project-type="operation"
          />
        </UiScrollArea>
      </UiPopover>
      <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
      <ToolbarButton as-child
        ><UiIconButton :label="t('common.close')" size="sm" @click="close"
          ><X class="size-3.5" /></UiIconButton
      ></ToolbarButton>
    </ToolbarRoot>
  </div>
</template>
