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
import SpaceCanvasFeedback from "./SpaceCanvasFeedback.vue";
import SpaceCanvasFocusBadge from "./SpaceCanvasFocusBadge.vue";
import SpaceCanvasRelationBar from "./SpaceCanvasRelationBar.vue";
import SpaceCanvasSceneControls from "./SpaceCanvasSceneControls.vue";
import SpaceCanvasSelectionAnchor from "./SpaceCanvasSelectionAnchor.vue";
import SpaceCanvasSelectionBar from "./SpaceCanvasSelectionBar.vue";
import SpaceCanvasToolbar from "./SpaceCanvasToolbar.vue";
import SpaceCanvasViewControls from "./SpaceCanvasViewControls.vue";

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
const scenesCollapsed = shallowRef(false);
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const selection = computed(() => selectedNodes.value.filter((node) => node.data.kind !== "section"));
const derivations = useSelectionDerivations({ store, selection });
const emit = defineEmits<{
  dismissError: [];
  dismissCapture: [id: string];
  retryCapture: [capture: SpaceUrlCaptureProjection];
}>();
const canvasChromeInsets = { top: 56, right: 16, bottom: 56, left: 16 } as const;
</script>

<template>
  <div class="pointer-events-none absolute inset-0 z-30 overflow-hidden">
    <SpaceCanvasToolbar class="pointer-events-auto absolute top-4 left-1/2 -translate-x-1/2" />
    <SpaceCanvasFocusBadge
      v-if="store.canvasFocusIds.length"
      class="pointer-events-auto absolute top-14 left-4"
      :count="store.canvasFocusIds.length"
      @exit="store.clearCanvasFocus"
    />
    <div
      v-if="!empty"
      class="pointer-events-auto absolute bottom-1.5 left-1.5 flex h-7 items-center overflow-hidden rounded-control border border-border-strong bg-canvas"
    >
      <SpaceCanvasViewControls />
      <Transition
        enter-active-class="transition-[opacity,transform] duration-150 ease-standard motion-reduce:transition-none"
        enter-from-class="-translate-x-1 opacity-0"
        leave-active-class="transition-[opacity,transform] duration-100 ease-standard motion-reduce:transition-none"
        leave-to-class="-translate-x-1 opacity-0"
      >
        <div
          v-if="!scenesCollapsed"
          class="flex h-full w-17 shrink-0 items-center border-x border-border-strong bg-control px-0.5"
        >
          <SpaceCanvasSceneControls />
        </div>
      </Transition>
      <UiIconButton
        :label="t(scenesCollapsed ? 'spaces.navigation.expandScenes' : 'spaces.navigation.collapseScenes')"
        size="sm"
        class="mx-0.5 size-5 rounded-grid-cell border border-border-strong text-foreground hover:border-border-hover hover:bg-control-hover"
        @click="scenesCollapsed = !scenesCollapsed"
      >
        <Star
          class="size-(--knovum-icon-size-xs) transition-transform duration-150 ease-standard"
          :class="scenesCollapsed ? 'scale-95' : 'scale-100'"
          :stroke-width="1.55"
        />
      </UiIconButton>
    </div>
    <SpaceCanvasSelectionAnchor
      v-if="
        tools.tool === 'select' &&
        !editorSession.directManipulation.value &&
        !editorSession.activeTextSelectionNodeId.value
      "
      :bounds="editorSession.selection.value.bounds"
      :canvas-to-viewport-point="canvasToViewportPoint"
      :safe-area="canvasChromeInsets"
    >
      <SpaceCanvasRelationBar v-if="editorSession.selection.value.relationIds.length === 1" />
      <SpaceCanvasSelectionBar
        v-else-if="editorSession.selection.value.nodeIds.length"
        :derivations="derivations.decisions.value"
        @invoke-derivation="derivations.run"
      />
    </SpaceCanvasSelectionAnchor>
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
      @social="derivations.submitSocial"
      @meeting="derivations.submitMeetingPackage"
      @presentation="derivations.submitPresentation"
    />
    <SpaceFind v-model:open="store.findOpen" />
    <SpaceGlobalFind v-model:open="store.globalFindOpen" />
  </div>
</template>
