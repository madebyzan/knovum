<script setup lang="ts">
import { ChevronsLeftRight } from "@lucide/vue";
import { onKeyStroke, useElementSize } from "@vueuse/core";
import { clamp } from "es-toolkit";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import {
  ComparisonFrame,
  sameView,
  type ComparisonSize,
  type ComparisonView,
} from "../comparison/comparison-view";
import SpaceImageComparisonZoom from "./SpaceImageComparisonZoom.vue";

/**
 * Wipes between a derived picture and the input it was made from. Both are drawn into the output's
 * frame — the transforms that compare keep the frame — so a pixel of one sits on the same point of
 * the other. It opens at actual size from the centre, where the difference is visible; fitted, an
 * upscale and its input look alike. The divider stays put on screen while the picture moves under it.
 */
const props = defineProps<{ before: string; after: string }>();
const { t } = useI18n();
const stage = useTemplateRef<HTMLElement>("stage");
const { width, height } = useElementSize(stage);
const picture = shallowRef<ComparisonSize>();
const view = shallowRef<ComparisonView>();
/** The divider's place, as a share of the stage's width. */
const split = shallowRef(0.5);
const dragging = shallowRef<"split" | "pan">();
let last = { x: 0, y: 0 };

const frame = computed(() =>
  picture.value && width.value > 0 && height.value > 0
    ? new ComparisonFrame({ width: width.value, height: height.value }, picture.value)
    : undefined,
);
/** First placement opens at actual size; a resized stage keeps the view, re-bounded. */
watch(frame, (next) => {
  if (next) view.value = view.value ? next.zoomed(view.value, { x: 0, y: 0 }, 1) : next.actualSize;
});

const span = computed(() => (frame.value && view.value ? frame.value.span(view.value) : undefined));
/** Where the divider is drawn: kept on the picture, never in the backdrop beside it. */
const dividerX = computed(() =>
  span.value ? clamp(split.value * width.value, span.value.left, span.value.right) : 0,
);
const pictureStyle = computed(() => {
  if (!view.value || !picture.value) return { visibility: "hidden" as const };
  const { scale, x, y } = view.value;
  return {
    width: `${picture.value.width}px`,
    height: `${picture.value.height}px`,
    transform: `translate(${x}px, ${y}px) scale(${scale})`,
  };
});
/** The two presets a view can be at, and whether it is at each; zoom's reach either way. */
const presets = computed(() => {
  if (!frame.value || !view.value) return undefined;
  return {
    fitted: sameView(view.value, frame.value.fitted),
    actualSize: sameView(view.value, frame.value.actualSize),
    zoomOut: frame.value.canZoomOut(view.value),
    zoomIn: frame.value.canZoomIn(view.value),
  };
});

function show(preset: "fitted" | "actualSize") {
  if (frame.value) view.value = frame.value[preset];
}

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
  if (dragging.value === "split") split.value = clamp(stagePoint(event).x / width.value, 0, 1);
  else if (dragging.value === "pan" && frame.value && view.value) {
    view.value = frame.value.panned(view.value, event.clientX - last.x, event.clientY - last.y);
    last = { x: event.clientX, y: event.clientY };
  }
}

function zoom(event: WheelEvent) {
  if (frame.value && view.value)
    view.value = frame.value.zoomed(view.value, stagePoint(event), Math.exp(-event.deltaY * 0.0015));
}

/** Double-clicking toggles between the whole picture and actual size at the pointer. */
function toggle(event: MouseEvent) {
  if (!frame.value || !view.value) return;
  view.value = presets.value?.fitted
    ? frame.value.zoomed(view.value, stagePoint(event), 1 / view.value.scale)
    : frame.value.fitted;
}

function step(direction: 1 | -1) {
  if (frame.value && view.value) view.value = frame.value.stepped(view.value, direction);
}

function nudge(event: KeyboardEvent) {
  const direction = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
  if (!direction || !span.value) return;
  event.preventDefault();
  split.value = clamp(dividerX.value / width.value + direction * 0.05, 0, 1);
}

onKeyStroke(["+", "="], () => step(1), { dedupe: false });
onKeyStroke("-", () => step(-1), { dedupe: false });
onKeyStroke("0", () => show("fitted"));
onKeyStroke("1", () => show("actualSize"));
</script>

<template>
  <div
    ref="stage"
    class="absolute inset-0 touch-none overflow-hidden select-none"
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
    <div class="absolute inset-0" :style="{ clipPath: `inset(0 ${width - dividerX}px 0 0)` }">
      <div class="absolute top-0 left-0 origin-top-left" :style="pictureStyle">
        <img :src="props.before" alt="" draggable="false" class="size-full" />
      </div>
    </div>
    <div
      v-if="span"
      data-comparison-divider
      class="absolute flex w-8 -translate-x-1/2 cursor-ew-resize justify-center"
      :style="{ left: `${dividerX}px`, top: `${span.top}px`, height: `${span.bottom - span.top}px` }"
    >
      <span class="h-full w-0.5 bg-white shadow-[0_0_0_1px_rgb(0_0_0/0.28),0_0_10px_rgb(0_0_0/0.3)]" />
      <span
        role="slider"
        tabindex="0"
        aria-orientation="horizontal"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="Math.round((dividerX / width) * 100)"
        :aria-label="t('spaces.media.imageComparison.divider')"
        class="absolute top-1/2 left-1/2 grid size-9 -translate-1/2 place-items-center rounded-full bg-white text-neutral-900 shadow-[0_0_0_1px_rgb(0_0_0/0.12),0_4px_14px_rgb(0_0_0/0.35)] transition-transform outline-none hover:scale-105 focus-visible:ring-2 focus-visible:ring-white focus-visible:ring-offset-2 focus-visible:ring-offset-neutral-900"
        @keydown="nudge"
      >
        <ChevronsLeftRight class="size-4" :stroke-width="2.2" />
      </span>
    </div>
  </div>
  <template v-if="presets">
    <span
      class="pointer-events-none absolute bottom-5 left-5 rounded-md bg-neutral-900/80 px-2 py-1 text-caption-sm font-medium text-white ring-1 ring-white/10 backdrop-blur-md"
    >
      {{ t("spaces.media.imageComparison.before") }}
    </span>
    <span
      class="pointer-events-none absolute right-5 bottom-5 rounded-md bg-neutral-900/80 px-2 py-1 text-caption-sm font-medium text-white ring-1 ring-white/10 backdrop-blur-md"
    >
      {{ t("spaces.media.imageComparison.after") }}
    </span>
    <SpaceImageComparisonZoom :scale="view?.scale ?? 1" :presets="presets" @step="step" @show="show" />
  </template>
</template>
