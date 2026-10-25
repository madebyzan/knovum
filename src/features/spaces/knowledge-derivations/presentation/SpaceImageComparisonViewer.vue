<script setup lang="ts">
import { ChevronsLeftRight } from "@lucide/vue";
import { useElementSize } from "@vueuse/core";
import { clamp } from "es-toolkit";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import {
  actualSizeView,
  fittedView,
  zoomedView,
  type ComparisonSize,
  type ComparisonView,
} from "../comparison/comparison-view";

/**
 * Lays a derived picture over the input it was made from, at the files' own resolution. Both are
 * drawn into the output's frame — the transforms that compare keep the frame — so a pixel of one
 * sits on the same point of the other, and an upscale's input shows exactly as soft as it was.
 */
const props = defineProps<{ before: string; after: string }>();
const { t } = useI18n();
const stage = useTemplateRef<HTMLElement>("stage");
const { width, height } = useElementSize(stage);
const picture = shallowRef<ComparisonSize>();
const view = shallowRef<ComparisonView>();
const split = shallowRef(0.5);
const dragging = shallowRef<"split" | "pan">();
let last = { x: 0, y: 0 };

const fitted = computed(() =>
  picture.value && width.value > 0 && height.value > 0
    ? fittedView({ width: width.value, height: height.value }, picture.value)
    : undefined,
);
watch(fitted, (next) => (view.value = next));

const pictureStyle = computed(() => {
  if (!view.value || !picture.value) return { visibility: "hidden" as const };
  const { scale, x, y } = view.value;
  return {
    width: `${picture.value.width}px`,
    height: `${picture.value.height}px`,
    transform: `translate(${x}px, ${y}px) scale(${scale})`,
  };
});
const zoomLabel = computed(() => `${Math.round((view.value?.scale ?? 1) * 100)}%`);

function measure(event: Event) {
  const image = event.target as HTMLImageElement;
  picture.value = { width: image.naturalWidth, height: image.naturalHeight };
}

function stagePoint(event: MouseEvent) {
  const box = stage.value!.getBoundingClientRect();
  return { x: event.clientX - box.left, y: event.clientY - box.top };
}

function begin(event: PointerEvent) {
  if (event.button !== 0) return;
  dragging.value = (event.target as Element).closest("[data-comparison-divider]") ? "split" : "pan";
  last = { x: event.clientX, y: event.clientY };
  stage.value?.setPointerCapture(event.pointerId);
  move(event);
}

function move(event: PointerEvent) {
  if (dragging.value === "split") {
    const box = stage.value!.getBoundingClientRect();
    split.value = clamp((event.clientX - box.left) / box.width, 0, 1);
  } else if (dragging.value === "pan" && view.value) {
    view.value = {
      ...view.value,
      x: view.value.x + event.clientX - last.x,
      y: view.value.y + event.clientY - last.y,
    };
    last = { x: event.clientX, y: event.clientY };
  }
}

function zoom(event: WheelEvent) {
  if (view.value && fitted.value)
    view.value = zoomedView(view.value, stagePoint(event), Math.exp(-event.deltaY * 0.0015), fitted.value);
}

/** Double-clicking toggles between the whole picture and its own resolution at the pointer. */
function toggle(event: MouseEvent) {
  if (!view.value || !fitted.value) return;
  view.value =
    view.value.scale > fitted.value.scale
      ? fitted.value
      : zoomedView(view.value, stagePoint(event), 1 / view.value.scale, fitted.value);
}

function showActualSize() {
  if (picture.value) view.value = actualSizeView({ width: width.value, height: height.value }, picture.value);
}

function nudge(event: KeyboardEvent) {
  const step = event.key === "ArrowLeft" ? -0.05 : event.key === "ArrowRight" ? 0.05 : 0;
  if (!step) return;
  event.preventDefault();
  split.value = clamp(split.value + step, 0, 1);
}
</script>

<template>
  <div class="relative size-full">
    <div
      ref="stage"
      class="relative size-full touch-none overflow-hidden bg-surface-canvas select-none"
      :class="dragging === 'pan' ? 'cursor-grabbing' : 'cursor-grab'"
      @pointerdown="begin"
      @pointermove="move"
      @pointerup="dragging = undefined"
      @pointercancel="dragging = undefined"
      @wheel.prevent="zoom"
      @dblclick="toggle"
    >
      <div class="absolute top-0 left-0 origin-top-left" :style="pictureStyle">
        <img :src="props.after" alt="" draggable="false" class="size-full" @load="measure" />
      </div>
      <div class="absolute inset-0" :style="{ clipPath: `inset(0 ${(1 - split) * 100}% 0 0)` }">
        <div class="absolute top-0 left-0 origin-top-left" :style="pictureStyle">
          <img :src="props.before" alt="" draggable="false" class="size-full" />
        </div>
      </div>
      <span
        class="pointer-events-none absolute top-3 left-3 rounded-full bg-surface-inverse/75 px-2.5 py-1 text-caption font-medium text-fg-inverse"
        >{{ t("spaces.media.imageComparison.before") }}</span
      >
      <span
        class="pointer-events-none absolute top-3 right-3 rounded-full bg-surface-inverse/75 px-2.5 py-1 text-caption font-medium text-fg-inverse"
        >{{ t("spaces.media.imageComparison.after") }}</span
      >
      <div
        data-comparison-divider
        class="absolute inset-y-0 flex w-6 -translate-x-1/2 cursor-ew-resize justify-center"
        :style="{ left: `${split * 100}%` }"
      >
        <span class="h-full w-px bg-surface-panel shadow-popover" />
        <span
          role="slider"
          tabindex="0"
          aria-orientation="horizontal"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-valuenow="Math.round(split * 100)"
          :aria-label="t('spaces.media.imageComparison.divider')"
          class="absolute top-1/2 left-1/2 grid size-8 -translate-1/2 place-items-center rounded-full bg-surface-inverse text-fg-inverse shadow-popover outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
          @keydown="nudge"
        >
          <ChevronsLeftRight class="size-4" :stroke-width="1.8" />
        </span>
      </div>
    </div>
    <div
      class="absolute bottom-3 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-lg border border-stroke-default bg-surface-panel/95 p-1 shadow-popover"
    >
      <span class="min-w-12 px-2 text-center text-caption text-fg-secondary tabular-nums">{{
        zoomLabel
      }}</span>
      <UiButton size="sm" variant="ghost" @click="view = fitted">{{
        t("spaces.media.imageComparison.fit")
      }}</UiButton>
      <UiButton size="sm" variant="ghost" @click="showActualSize">{{
        t("spaces.media.imageComparison.actualSize")
      }}</UiButton>
    </div>
  </div>
</template>
