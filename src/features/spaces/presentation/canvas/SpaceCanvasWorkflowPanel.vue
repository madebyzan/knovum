<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CanvasPoint } from "../../canvas/contracts/canvas-editor-port";
import { getCanvasEditorSession, useCanvasSelectionProjection } from "../../canvas/editor-session";
import { useSpaceStore } from "../../session/space-store";
import SpaceContextHeader from "../SpaceContextHeader.vue";
import SpaceWorkflowCapture from "../workflows/SpaceWorkflowCapture.vue";
import SpaceWorkflowPicker from "../workflows/SpaceWorkflowPicker.vue";
import type { SurfaceRect } from "./selection-surface-geometry";
import SpaceCanvasSelectionAnchor from "./SpaceCanvasSelectionAnchor.vue";

defineProps<{
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
  safeArea: { top: number; right: number; bottom: number; left: number };
  reserved: readonly SurfaceRect[];
}>();

const store = useSpaceStore();
const editor = getCanvasEditorSession();
const panel = useTemplateRef<HTMLElement>("panel");
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const { t } = useI18n();
const selectionFingerprint = () =>
  selectedNodes.value
    .map(({ id }) => id)
    .toSorted()
    .join("\u0000");
const openingSelection = selectionFingerprint();
const title = computed(() =>
  t(
    store.workflowMode === "capture"
      ? "spaces.creation.workflow.createFromSelection"
      : "spaces.creation.workflow.useSaved",
  ),
);

function close() {
  store.workflowMode = undefined;
}

watch(selectionFingerprint, (ids) => {
  if (ids !== openingSelection) close();
});
watch(() => store.activeSpaceId, close);
function focusPanel() {
  panel.value?.focus({ preventScroll: true });
}
</script>

<template>
  <SpaceCanvasSelectionAnchor
    :bounds="editor.selection.value.bounds"
    :members="editor.selection.value.members"
    :canvas-to-viewport-point="canvasToViewportPoint"
    :safe-area="safeArea"
    :reserved="reserved"
    :selection-key="openingSelection"
    kind="inspector"
    :gap="16"
    @ready="focusPanel"
  >
    <template #default="{ availableHeight, availableWidth }">
      <section
        ref="panel"
        role="region"
        :aria-label="title"
        tabindex="-1"
        class="flex h-[min(34rem,calc(100vh-7rem))] w-[min(23.75rem,calc(100vw-2rem))] min-w-0 flex-col overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-panel shadow-panel"
        :style="{
          maxHeight: availableHeight ? `${availableHeight}px` : undefined,
          maxWidth: `${availableWidth}px`,
        }"
        @keydown.esc.stop="close"
      >
        <SpaceContextHeader :title="title" closable @close="close" />
        <div class="min-h-0 flex-1 overflow-hidden">
          <SpaceWorkflowCapture v-if="store.workflowMode === 'capture'" class="h-full" />
          <SpaceWorkflowPicker v-else class="h-full" />
        </div>
      </section>
    </template>
  </SpaceCanvasSelectionAnchor>
</template>
