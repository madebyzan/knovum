<script setup lang="ts">
import { Star } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../../canvas/editor-session";
import { useSelectionDerivations } from "../../knowledge-derivations/selection/use-selection-derivations";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceUrlCaptureProjection } from "../../sources/url-capture-projection";
import SpaceDerivationDialogs from "../derivations/SpaceDerivationDialogs.vue";
import SpaceFind from "../SpaceFind.vue";
import SpaceGlobalFind from "../SpaceGlobalFind.vue";
import { canvasSelectionGap } from "./canvas-selection-gap";
import SpaceCanvasFeedback from "./SpaceCanvasFeedback.vue";
import SpaceCanvasFocusBadge from "./SpaceCanvasFocusBadge.vue";
import SpaceCanvasRelationBar from "./SpaceCanvasRelationBar.vue";
import SpaceCanvasSceneControls from "./SpaceCanvasSceneControls.vue";
import SpaceCanvasSelectionAnchor from "./SpaceCanvasSelectionAnchor.vue";
import SpaceCanvasSelectionBar from "./SpaceCanvasSelectionBar.vue";
import SpaceCanvasToolbar from "./SpaceCanvasToolbar.vue";
import SpaceCanvasViewControls from "./SpaceCanvasViewControls.vue";
import SpaceCanvasWorkflowPanel from "./SpaceCanvasWorkflowPanel.vue";
import { useCanvasChromeRects } from "./use-canvas-chrome-rects";
import { useSelectionSurfaceState } from "./use-selection-surface-state";
defineProps<{
  canvasToViewportPoint(point: { x: number; y: number }): { x: number; y: number };
  captures: readonly SpaceUrlCaptureProjection[];
  empty: boolean;
  error?: string;
  fileHover: boolean;
  fileHoverPosition?: { x: number; y: number };
  spaceId: string;
}>();
const store = useSpaceStore();
const tools = useCanvasToolStore();
const { t } = useI18n();
const editorSession = useCanvasEditorSession();
const {
  angleNodeId,
  selectionKey,
  kind: selectionSurfaceKind,
  showActions: showSelectionActions,
} = useSelectionSurfaceState();
const scenesCollapsed = shallowRef(false);
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const derivations = useSelectionDerivations({ store, selection: computed(() => selectedNodes.value) });
const emit = defineEmits<{
  dismissError: [];
  dismissCapture: [id: string];
  retryCapture: [capture: SpaceUrlCaptureProjection];
}>();
const canvasChromeInsets = { top: 8, right: 16, bottom: 8, left: 16 } as const;
const { chromeRects } = useCanvasChromeRects();
const singleCanvasObjectSelected = computed(
  () =>
    editorSession.selection.value.nodeIds.length + editorSession.selection.value.structureNodeIds.length ===
    1,
);
const selectionBarGap = computed(() =>
  canvasSelectionGap(selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined),
);
</script>
<template>
  <div ref="chromeRoot" class="pointer-events-none absolute inset-0 z-30 overflow-hidden">
    <div ref="topChrome" class="pointer-events-auto absolute top-4 left-1/2 -translate-x-1/2">
      <SpaceCanvasToolbar />
    </div>
    <div
      v-if="store.canvasFocusIds.length"
      ref="focusChrome"
      class="pointer-events-auto absolute top-14 left-4"
    >
      <SpaceCanvasFocusBadge :count="store.canvasFocusIds.length" @exit="store.clearCanvasFocus" />
    </div>
    <div
      v-if="!empty"
      ref="viewChrome"
      class="pointer-events-auto absolute bottom-1 left-1 flex h-6 items-center overflow-hidden rounded-md border border-stroke-strong bg-surface-canvas"
    >
      <SpaceCanvasViewControls />
      <Transition
        enter-active-class="overflow-hidden transition-[width,opacity,transform] duration-220 ease-standard motion-reduce:transition-none"
        enter-from-class="w-0 -translate-x-1 opacity-0"
        leave-active-class="overflow-hidden transition-[width,opacity,transform] duration-150 ease-standard motion-reduce:transition-none"
        leave-to-class="w-0 -translate-x-1 opacity-0"
      >
        <div
          v-if="!scenesCollapsed"
          class="flex h-full w-17 shrink-0 items-center overflow-hidden border-l border-stroke-strong bg-surface-control px-0.5"
        >
          <SpaceCanvasSceneControls />
        </div>
      </Transition>
      <div class="flex h-full shrink-0 items-center border-l border-stroke-strong px-0.5">
        <UiIconButton
          :label="t(scenesCollapsed ? 'spaces.scenes.expand' : 'spaces.scenes.collapse')"
          tooltip
          size="sm"
          class="size-5 rounded-sm text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
          @click="scenesCollapsed = !scenesCollapsed"
        >
          <Star class="size-3" :stroke-width="1.55" />
        </UiIconButton>
      </div>
    </div>
    <SpaceCanvasSelectionAnchor
      v-if="tools.tool === 'select' && showSelectionActions"
      :bounds="editorSession.selection.value.bounds"
      :members="editorSession.selection.value.members"
      :canvas-to-viewport-point="canvasToViewportPoint"
      :safe-area="canvasChromeInsets"
      :reserved="chromeRects"
      :selection-key="selectionKey"
      :kind="selectionSurfaceKind"
      :gap="selectionSurfaceKind === 'actions' ? (singleCanvasObjectSelected ? selectionBarGap : 34) : 12"
      v-slot="{ availableWidth }"
    >
      <SpaceCanvasRelationBar v-if="editorSession.selection.value.relationIds.length === 1" />
      <SpaceCanvasSelectionBar
        v-else-if="
          editorSession.selection.value.nodeIds.length ||
          editorSession.selection.value.structureNodeIds.length
        "
        :derivations="derivations.decisions.value"
        :available-width="availableWidth"
        :angle-node-id="angleNodeId"
        @update:angle-node-id="angleNodeId = $event"
        @invoke-derivation="derivations.run"
      />
    </SpaceCanvasSelectionAnchor>
    <SpaceCanvasWorkflowPanel
      v-if="store.workflowMode"
      :canvas-to-viewport-point="canvasToViewportPoint"
      :safe-area="canvasChromeInsets"
      :reserved="chromeRects"
    />
    <SpaceCanvasFeedback
      :canvas-to-viewport-point="canvasToViewportPoint"
      :captures="captures"
      :empty="empty"
      :file-hover="fileHover"
      :space-id="spaceId"
      v-bind="{
        ...(error ? { error } : {}),
        ...(fileHoverPosition ? { fileHoverPosition } : {}),
      }"
      @dismiss-error="emit('dismissError')"
      @dismiss-capture="emit('dismissCapture', $event)"
      @retry-capture="emit('retryCapture', $event)"
    />
    <SpaceDerivationDialogs
      v-model:open="derivations.dialog.value"
      :source-title="derivations.sourceTitle.value"
      :sources="derivations.sources.value ?? []"
      @document="derivations.submitDocument"
      @visualization="derivations.submitVisualization"
      @video="derivations.submitVideo"
    />
    <SpaceFind v-model:open="store.findOpen" />
    <SpaceGlobalFind v-model:open="store.globalFindOpen" />
  </div>
</template>
