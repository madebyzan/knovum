<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CanvasPoint } from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../../canvas/editor-session";
import { useSpaceStore } from "../../session/space-store";
import SpaceContextHeader from "../SpaceContextHeader.vue";
import SpaceWorkflowCapture from "../workflows/SpaceWorkflowCapture.vue";
import SpaceWorkflowPicker from "../workflows/SpaceWorkflowPicker.vue";
import SpaceCanvasSelectionAnchor from "./SpaceCanvasSelectionAnchor.vue";

defineProps<{
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
  safeArea: { top: number; right: number; bottom: number; left: number };
}>();

const store = useSpaceStore();
const editor = useCanvasEditorSession();
const panel = useTemplateRef<HTMLElement>("panel");
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const { t } = useI18n();
const openingSelection = selectedNodes.value
  .map(({ id }) => id)
  .toSorted()
  .join("\u0000");
const title = computed(() =>
  t(
    store.inspectorMode === "workflow"
      ? "spaces.creation.createWorkflowFromSelection"
      : "spaces.creation.useSavedWorkflow",
  ),
);

function close() {
  store.setInspectorMode("inspect");
}

watch(
  () =>
    selectedNodes.value
      .map(({ id }) => id)
      .toSorted()
      .join("\u0000"),
  (ids) => {
    if (ids !== openingSelection) close();
  },
);
watch(() => store.activeSpaceId, close);
function focusPanel() {
  panel.value?.focus({ preventScroll: true });
}
</script>

<template>
  <SpaceCanvasSelectionAnchor
    :bounds="editor.selection.value.bounds"
    :canvas-to-viewport-point="canvasToViewportPoint"
    :safe-area="safeArea"
    kind="inspector"
    :gap="16"
    @ready="focusPanel"
  >
    <template #default="{ availableHeight }">
      <section
        ref="panel"
        role="region"
        :aria-label="title"
        tabindex="-1"
        class="flex h-[min(34rem,calc(100vh-7rem))] w-[min(23.75rem,calc(100vw-2rem))] min-w-0 flex-col overflow-hidden rounded-workspace border border-window-hairline bg-panel shadow-panel"
        :style="availableHeight ? { maxHeight: `${availableHeight}px` } : undefined"
        @keydown.esc.stop="close"
      >
        <SpaceContextHeader :title="title" :window-drag-region="false" @close="close" />
        <div class="min-h-0 flex-1 overflow-hidden">
          <SpaceWorkflowCapture v-if="store.inspectorMode === 'workflow'" class="h-full" />
          <SpaceWorkflowPicker v-else class="h-full" />
        </div>
      </section>
    </template>
  </SpaceCanvasSelectionAnchor>
</template>
