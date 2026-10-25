<script setup lang="ts">
import { ChevronsLeftRight, Shrink, X } from "@lucide/vue";
import { useElementSize } from "@vueuse/core";
import { clamp } from "es-toolkit";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { DIALOG_LAYER_CLASSES } from "../../../../shared/ui/ui-classes";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import {
  actualSizeView,
  fittedView,
  zoomedView,
  type ComparisonSize,
  type ComparisonView,
} from "../comparison/comparison-view";

/**
 * Lays a derived picture over the input it was made from, full window and at the files' own
 * resolution. Both are drawn into the output's frame — the transforms that compare keep the frame —
 * so a pixel of one sits on the same point of the other. The divider stays where it is on screen
 * while the picture pans beneath it, so a detail can be wiped across without chasing the handle.
 */
const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ before: string | undefined; after: string; error: unknown }>();
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
/** The divider spans the picture, not the empty stage around it. */
const dividerStyle = computed(() => {
  const top = view.value ? clamp(view.value.y, 0, height.value) : 0;
  const bottom =
    view.value && picture.value
      ? clamp(view.value.y + picture.value.height * view.value.scale, 0, height.value)
      : height.value;
  return { left: `${split.value * 100}%`, top: `${top}px`, height: `${bottom - top}px` };
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
    split.value = clamp(stagePoint(event).x / width.value, 0, 1);
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
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay
        class="fixed inset-0 bg-black/90 duration-150 ease-standard data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
        :class="DIALOG_LAYER_CLASSES.dialog.overlay"
      />
      <DialogContent
        class="fixed inset-0 text-white outline-none data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
        :class="DIALOG_LAYER_CLASSES.dialog.content"
      >
        <DialogTitle class="sr-only">{{ t("spaces.media.imageComparison.title") }}</DialogTitle>
        <DialogDescription class="sr-only">{{ t("spaces.media.imageComparison.hint") }}</DialogDescription>
        <div
          v-if="props.before"
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
          <div class="absolute inset-0" :style="{ clipPath: `inset(0 ${(1 - split) * 100}% 0 0)` }">
            <div class="absolute top-0 left-0 origin-top-left" :style="pictureStyle">
              <img :src="props.before" alt="" draggable="false" class="size-full" />
            </div>
          </div>
          <div
            v-if="view"
            data-comparison-divider
            class="absolute flex w-6 -translate-x-1/2 cursor-ew-resize justify-center"
            :style="dividerStyle"
          >
            <span class="h-full w-0.5 bg-white/90 shadow-[0_0_6px_rgb(0_0_0/0.4)]" />
            <span
              role="slider"
              tabindex="0"
              aria-orientation="horizontal"
              aria-valuemin="0"
              aria-valuemax="100"
              :aria-valuenow="Math.round(split * 100)"
              :aria-label="t('spaces.media.imageComparison.divider')"
              class="absolute top-1/2 left-1/2 grid size-8 -translate-1/2 place-items-center rounded-full bg-white text-black shadow-dialog outline-none focus-visible:ring-2 focus-visible:ring-white/60"
              @keydown="nudge"
            >
              <ChevronsLeftRight class="size-4" :stroke-width="2" />
            </span>
          </div>
        </div>
        <p v-else class="grid size-full place-items-center text-caption text-white/60">
          {{ props.error ? getErrorMessage(props.error) : t("spaces.media.imageComparison.preparing") }}
        </p>
        <DialogClose
          :aria-label="t('common.close')"
          class="absolute top-4 right-4 grid size-9 place-items-center rounded-full bg-white/10 text-white/85 transition-colors hover:bg-white/20 hover:text-white focus-visible:ring-2 focus-visible:ring-white/60 focus-visible:outline-none"
        >
          <X class="size-4.5" :stroke-width="1.8" />
        </DialogClose>
        <template v-if="view">
          <span
            class="pointer-events-none absolute bottom-5 left-5 rounded-full bg-black/55 px-2.5 py-1 text-caption-sm font-medium text-white/90 backdrop-blur-sm"
            >{{ t("spaces.media.imageComparison.before") }}</span
          >
          <span
            class="pointer-events-none absolute right-5 bottom-5 rounded-full bg-black/55 px-2.5 py-1 text-caption-sm font-medium text-white/90 backdrop-blur-sm"
            >{{ t("spaces.media.imageComparison.after") }}</span
          >
          <div
            class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-0.5 rounded-full bg-black/55 p-1 text-caption-sm text-white/85 backdrop-blur-sm"
          >
            <span class="min-w-11 px-1.5 text-center tabular-nums">{{ zoomLabel }}</span>
            <UiTooltip :text="t('spaces.media.imageComparison.fit')" side="top">
              <button
                type="button"
                :aria-label="t('spaces.media.imageComparison.fit')"
                class="grid size-7 place-items-center rounded-full transition-colors hover:bg-white/15 hover:text-white focus-visible:ring-2 focus-visible:ring-white/60 focus-visible:outline-none"
                @click="view = fitted"
              >
                <Shrink class="size-3.5" :stroke-width="1.8" />
              </button>
            </UiTooltip>
            <UiTooltip :text="t('spaces.media.imageComparison.actualSize')" side="top">
              <button
                type="button"
                :aria-label="t('spaces.media.imageComparison.actualSize')"
                class="h-7 rounded-full px-2 font-medium tabular-nums transition-colors hover:bg-white/15 hover:text-white focus-visible:ring-2 focus-visible:ring-white/60 focus-visible:outline-none"
                @click="showActualSize"
              >
                1:1
              </button>
            </UiTooltip>
          </div>
        </template>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
