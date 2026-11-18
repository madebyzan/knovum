<script setup lang="ts">
import { Map as MapIcon, Minimize2 } from "@lucide/vue";
import { useDevicePixelRatio } from "@vueuse/core";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import { SPACE_COMMAND_SHORTCUTS } from "../../../commands/space-command-shortcuts";
import type { SpaceRelation } from "../../../model/relations/relation";
import { useSpaceStore } from "../../../session/space-store";
import { getCanvasEditorSession } from "../../editor-session";
import { renderMinimapCanvas } from "../../minimap/minimap-renderer";
import { projectMinimapScene } from "../../minimap/minimap-scene";
import {
  computeMinimapTransform,
  computeMinimapViewport,
  isPointInMinimapRect,
  MINIMAP_DIMENSIONS,
} from "../../minimap/minimap-transform";
import { resolveMinimapVisibility } from "../../minimap/minimap-visibility";
import { useCanvasPalette } from "../../use-canvas-palette";
import { useMinimapNavigation } from "./use-minimap-navigation";

const props = defineProps<{
  navigationEnabled: boolean;
  hostSize: { readonly width: number; readonly height: number };
  relations: readonly SpaceRelation[];
}>();

const COLLAPSED_SIZE_PX = 24;

const { t } = useI18n();
const store = useSpaceStore();
const editorSession = getCanvasEditorSession();
const palette = useCanvasPalette();
const { pixelRatio } = useDevicePixelRatio();
const canvasEl = useTemplateRef<HTMLCanvasElement>("canvasEl");
const surface = useTemplateRef<HTMLElement>("surface");
const hoveringViewport = shallowRef(false);

const scene = computed(() => {
  if (!props.navigationEnabled) return undefined;
  return projectMinimapScene({
    snapshot: store.editSnapshot,
    nodes: store.nodes,
    relations: props.relations,
    selectedNodeIds: new Set(editorSession.selection.value.nodeIds),
    selectedStructureNodeIds: new Set(editorSession.selection.value.structureNodeIds),
    focusNodeIds: new Set(store.canvasFocusIds),
  });
});

const transform = computed(() =>
  scene.value?.contentBounds ? computeMinimapTransform(scene.value.contentBounds) : undefined,
);

const visibility = computed(() =>
  resolveMinimapVisibility({
    open: store.minimapOpen,
    navigationEnabled: props.navigationEnabled,
    hasContent: scene.value?.contentBounds !== undefined,
    hostSize: props.hostSize,
    recordingDockOpen: store.audioRecordingTarget !== undefined,
  }),
);

watch(
  () => visibility.value.showPanel,
  (open) => {
    if (!open) {
      hoveringViewport.value = false;
      navigation.cancel();
    }
  },
);

const shellStyle = computed(() => ({
  width: `${visibility.value.showPanel ? MINIMAP_DIMENSIONS.width : COLLAPSED_SIZE_PX}px`,
  height: `${visibility.value.showPanel ? MINIMAP_DIMENSIONS.height : COLLAPSED_SIZE_PX}px`,
}));

const viewportProjection = computed(() => {
  if (!visibility.value.showToggle || !editorSession.viewport.value || !transform.value) return undefined;
  return computeMinimapViewport(editorSession.viewport.value, props.hostSize, transform.value);
});

const viewportBoxStyle = computed(() => {
  const rect = viewportProjection.value?.rect;
  if (!rect) return undefined;
  return {
    transform: `translate3d(${rect.left}px, ${rect.top}px, 0)`,
    width: `${rect.width}px`,
    height: `${rect.height}px`,
  };
});

const offscreenBeaconStyle = computed(() => {
  const indicator = viewportProjection.value?.offscreenIndicator;
  if (!indicator) return undefined;
  return {
    transform: `translate3d(${indicator.x}px, ${indicator.y}px, 0) translate(-50%, -50%)`,
  };
});

const navigation = useMinimapNavigation({
  surface,
  editor: () => editorSession.activeEditor.value,
  viewport: () => editorSession.viewport.value,
  hostSize: () => props.hostSize,
  transform: () => transform.value,
  viewportProjection: () => viewportProjection.value,
});

watch(
  [canvasEl, () => visibility.value.showPanel, scene, transform, palette, pixelRatio],
  ([canvas, showPanel, currentScene, currentTransform, currentPalette, dpr]) => {
    if (!canvas || !showPanel || !currentScene || !currentTransform) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    renderMinimapCanvas(
      { ctx: context, transform: currentTransform, palette: currentPalette, dimensions: MINIMAP_DIMENSIONS },
      currentScene,
    );
  },
  { flush: "post", immediate: true },
);

function updateHoverCursor(event: PointerEvent) {
  if (navigation.dragging.value) return;
  const rect = viewportProjection.value?.rect;
  const element = surface.value;
  if (!rect || !element || !viewportProjection.value?.intersectsMap) {
    hoveringViewport.value = false;
    return;
  }
  const bounds = element.getBoundingClientRect();
  hoveringViewport.value = isPointInMinimapRect(
    { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
    rect,
  );
}

function onPointerMove(event: PointerEvent) {
  updateHoverCursor(event);
  navigation.onPointerMove(event);
}

function toggle() {
  store.minimapOpen = !store.minimapOpen;
}
</script>

<template>
  <div
    v-if="visibility.showToggle"
    class="group pointer-events-auto relative overflow-hidden border border-stroke-strong backdrop-blur-md transition-[width,height,border-radius,background-color,box-shadow] duration-220 ease-standard select-none motion-reduce:transition-none"
    :class="
      visibility.showPanel
        ? 'rounded-lg bg-surface-canvas/94 shadow-panel'
        : 'rounded-md bg-surface-canvas shadow-none'
    "
    :style="shellStyle"
  >
    <div
      class="absolute inset-0 grid place-items-center transition-[opacity,transform] duration-150 ease-standard motion-reduce:transition-none"
      :class="
        visibility.showPanel
          ? 'pointer-events-none scale-75 opacity-0'
          : 'pointer-events-auto scale-100 opacity-100 delay-75'
      "
      :aria-hidden="visibility.showPanel"
    >
      <UiIconButton
        :label="t('spaces.canvas.minimap.show')"
        :shortcut="SPACE_COMMAND_SHORTCUTS.toggleMinimap"
        :tabindex="visibility.showPanel ? -1 : 0"
        size="sm"
        class="size-5 rounded-sm focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:outline-none focus-visible:ring-inset"
        @click="toggle"
      >
        <MapIcon class="size-3" :stroke-width="1.55" />
      </UiIconButton>
    </div>

    <Transition
      enter-active-class="transition-opacity duration-200 ease-standard motion-reduce:transition-none"
      enter-from-class="opacity-0"
      leave-active-class="pointer-events-none transition-opacity duration-150 ease-standard motion-reduce:transition-none"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visibility.showPanel"
        ref="surface"
        class="absolute right-0 bottom-0 touch-none overflow-hidden"
        :class="
          navigation.dragging.value ? 'cursor-grabbing' : hoveringViewport ? 'cursor-grab' : 'cursor-pointer'
        "
        :style="{ width: `${MINIMAP_DIMENSIONS.width}px`, height: `${MINIMAP_DIMENSIONS.height}px` }"
        role="region"
        :aria-label="t('spaces.canvas.minimap.title')"
        @pointerdown.stop="navigation.onPointerDown"
        @pointermove="onPointerMove"
        @pointerleave="hoveringViewport = false"
        @pointerup="navigation.onPointerUp"
        @pointercancel="navigation.onPointerCancel"
        @wheel.stop="navigation.onWheel"
        @dblclick.stop="navigation.onDoubleClick"
        @contextmenu.prevent.stop
      >
        <canvas
          ref="canvasEl"
          :width="Math.round(MINIMAP_DIMENSIONS.width * pixelRatio)"
          :height="Math.round(MINIMAP_DIMENSIONS.height * pixelRatio)"
          class="pointer-events-none block size-full"
        />
        <div
          v-if="viewportProjection?.intersectsMap && viewportBoxStyle"
          class="pointer-events-none absolute top-0 left-0 rounded-[3px] border-[1.5px] border-stroke-accent transition-colors duration-100 ease-standard will-change-transform"
          :class="navigation.dragging.value ? 'bg-surface-accent/18' : 'bg-surface-accent/10'"
          :style="viewportBoxStyle"
          :aria-label="t('spaces.canvas.minimap.viewport')"
        />
        <div
          v-else-if="offscreenBeaconStyle"
          class="pointer-events-none absolute top-0 left-0 size-2.5 rounded-full bg-surface-accent ring-2 ring-surface-canvas"
          :style="offscreenBeaconStyle"
          :aria-label="t('spaces.canvas.minimap.viewport')"
        />
        <div class="absolute top-1 right-1" @pointerdown.stop @dblclick.stop>
          <UiIconButton
            :label="t('spaces.canvas.minimap.hide')"
            :shortcut="SPACE_COMMAND_SHORTCUTS.toggleMinimap"
            size="sm"
            class="size-5 rounded-sm bg-transparent text-fg-tertiary opacity-45 transition-[opacity,color,background-color] duration-150 ease-standard group-focus-within:opacity-100 group-hover:opacity-100 hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus focus-visible:outline-none focus-visible:ring-inset active:bg-surface-control-pressed motion-reduce:transition-none"
            @click="toggle"
          >
            <Minimize2 class="size-3" :stroke-width="1.65" />
          </UiIconButton>
        </div>
      </div>
    </Transition>
  </div>
</template>
