<script setup lang="ts">
import { ChevronsLeftRight, Maximize2 } from "@lucide/vue";
import { onBeforeUnmount, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import type { LightboxImageItem } from "../../../../shared/ui/image-group-layout";

defineProps<{
  before: LightboxImageItem;
  after: LightboxImageItem;
}>();

const emit = defineEmits<{ preview: [index: number] }>();
const { t } = useI18n();
const stageRef = useTemplateRef<HTMLElement>("stageRef");
const splitPercent = shallowRef(50);

let activePointerId: number | null = null;
let stageRect: { left: number; width: number } | null = null;
let latestClientX = 0;
let rafId: number | null = null;

function applySplit(percent: number) {
  const clamped = Math.max(2, Math.min(98, percent));
  splitPercent.value = Math.round(clamped);
  stageRef.value?.style.setProperty("--compare-split", `${clamped.toFixed(2)}%`);
}

function paintSplit() {
  rafId = null;
  if (!stageRect || stageRect.width <= 0) return;
  applySplit(((latestClientX - stageRect.left) / stageRect.width) * 100);
}

function stopDrag(event?: PointerEvent) {
  if (event && activePointerId !== null && event.pointerId !== activePointerId) return;
  if (rafId !== null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
  if (activePointerId !== null && stageRef.value?.hasPointerCapture?.(activePointerId)) {
    stageRef.value.releasePointerCapture(activePointerId);
  }
  activePointerId = null;
  stageRect = null;
}

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0 || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  const rect = stageRef.value.getBoundingClientRect();
  stageRect = { left: rect.left, width: rect.width };
  activePointerId = event.pointerId;
  latestClientX = event.clientX;
  stageRef.value.setPointerCapture?.(event.pointerId);
  paintSplit();
}

function onPointerMove(event: PointerEvent) {
  if (activePointerId === null || event.pointerId !== activePointerId) return;
  latestClientX = event.clientX;
  if (rafId === null) rafId = requestAnimationFrame(paintSplit);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    applySplit(splitPercent.value - 5);
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    applySplit(splitPercent.value + 5);
  }
}

onBeforeUnmount(() => stopDrag());
</script>

<template>
  <div
    ref="stageRef"
    data-testid="image-compare-stage"
    data-image-group-interactive
    tabindex="0"
    role="slider"
    :aria-label="t('spaces.richtext.image.group.compare')"
    aria-valuemin="2"
    aria-valuemax="98"
    :aria-valuenow="splitPercent"
    class="ui-image-compare group/compare outline-none focus-visible:ring-2 focus-visible:ring-fg-accent/70"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="stopDrag"
    @pointercancel="stopDrag"
    @dblclick.stop="applySplit(50)"
    @keydown.stop="onKeydown"
  >
    <img :src="after.src" :alt="after.caption" class="ui-image-compare-base" draggable="false" />
    <div class="ui-image-compare-overlay">
      <img :src="before.src" :alt="before.caption" draggable="false" />
    </div>
    <div class="ui-image-compare-divider" />
    <div class="ui-image-compare-knob">
      <ChevronsLeftRight class="size-4" :stroke-width="2" />
    </div>
    <span class="ui-image-compare-label" data-side="before">
      {{ before.caption.trim() || t("spaces.richtext.image.group.before") }}
    </span>
    <span class="ui-image-compare-label" data-side="after">
      {{ after.caption.trim() || t("spaces.richtext.image.group.after") }}
    </span>
    <button
      type="button"
      data-testid="image-compare-preview"
      :aria-label="t('spaces.richtext.image.preview')"
      :title="t('spaces.richtext.image.preview')"
      class="absolute top-3 right-3 grid size-7 place-items-center rounded-md bg-neutral-950/72 text-white/85 opacity-0 shadow-md ring-1 ring-white/12 transition-opacity duration-150 group-hover/compare:opacity-100 hover:bg-neutral-950/88 hover:text-white"
      @pointerdown.stop
      @click.stop="emit('preview', 0)"
    >
      <Maximize2 class="size-3.5" :stroke-width="1.65" />
    </button>
  </div>
</template>
