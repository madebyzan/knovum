<script setup lang="ts">
import { FileText, Image, Play, X } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getCanvasEditorSession } from "../../../canvas/editor-session";
import { generatorFrame, useCanvasGeneratorStore, type CanvasGenerator } from "../generator-store";

/**
 * Where an output will land, drawn at the size it will take. It moves with the canvas like the nodes
 * around it; its label and borders stay screen-sized so it reads the same at any zoom.
 */
const props = defineProps<{ generator: CanvasGenerator; active: boolean }>();
const { t } = useI18n();
const store = useCanvasGeneratorStore();
const editorSession = getCanvasEditorSession();
const ICONS = { image: Image, video: Play, document: FileText } as const;

const frame = computed(() => generatorFrame(props.generator));
const working = computed(() => props.generator.submission !== undefined);
const detail = computed(() => {
  const { kind, aspectRatio, durationSeconds } = props.generator;
  if (kind === "document") return undefined;
  return kind === "video" ? `${aspectRatio} · ${durationSeconds}s` : aspectRatio;
});
let drag: { pointerId: number; x: number; y: number; zoom: number } | undefined;

function press(event: PointerEvent) {
  store.activate(props.generator.id);
  if (event.button !== 0 || working.value) return;
  drag = {
    pointerId: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    zoom: editorSession.viewport.value?.zoom ?? 1,
  };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function follow(event: PointerEvent) {
  if (drag?.pointerId !== event.pointerId) return;
  const { x, y } = props.generator;
  store.move(
    props.generator.id,
    x + (event.clientX - drag.x) / drag.zoom,
    y + (event.clientY - drag.y) / drag.zoom,
  );
  drag = { ...drag, x: event.clientX, y: event.clientY };
}

function release(event: PointerEvent) {
  if (drag?.pointerId === event.pointerId) drag = undefined;
}
</script>

<template>
  <div
    data-canvas-generator
    class="pointer-events-auto absolute touch-none"
    :class="working ? 'cursor-default' : 'cursor-grab active:cursor-grabbing'"
    :style="{
      left: `${frame.x}px`,
      top: `${frame.y}px`,
      width: `${frame.width}px`,
      height: `${frame.height}px`,
    }"
    @pointerdown.stop="press"
    @pointermove="follow"
    @pointerup="release"
    @pointercancel="release"
  >
    <div class="generator-label flex items-center gap-1.5 text-caption text-fg-secondary">
      <component :is="ICONS[generator.kind]" class="size-3.5 shrink-0" :stroke-width="1.7" />
      <span class="min-w-0 truncate font-medium">{{
        t(`spaces.creation.generator.kinds.${generator.kind}`)
      }}</span>
      <span v-if="detail" class="ml-auto shrink-0 text-fg-tertiary tabular-nums">{{ detail }}</span>
      <button
        v-if="!working"
        type="button"
        class="grid size-5 shrink-0 place-items-center rounded-sm text-fg-tertiary hover:bg-surface-control-hover hover:text-fg-primary"
        :class="detail ? '' : 'ml-auto'"
        :aria-label="t('spaces.creation.generator.close')"
        @pointerdown.stop
        @click="store.close(generator.id)"
      >
        <X class="size-3.5" :stroke-width="1.8" />
      </button>
    </div>
    <div
      class="generator-frame grid size-full place-items-center bg-surface-muted"
      :data-active="active || undefined"
    >
      <div class="generator-mark flex flex-col items-center gap-2 text-fg-tertiary">
        <component
          :is="ICONS[generator.kind]"
          class="size-7"
          :class="working && 'animate-pulse'"
          :stroke-width="1.4"
        />
        <span v-if="working" class="text-caption">{{ t("spaces.creation.generator.generating") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* `--canvas-zoom` is published by the camera on the world layer: what is divided by it stays screen-sized. */
.generator-frame {
  border: calc(1px / var(--canvas-zoom)) solid var(--color-stroke-strong);
  border-radius: calc(10px / var(--canvas-zoom));
}
.generator-frame[data-active] {
  border-color: var(--color-stroke-accent);
  border-width: calc(2px / var(--canvas-zoom));
}
.generator-label {
  position: absolute;
  bottom: 100%;
  left: 0;
  width: calc(100% * var(--canvas-zoom));
  padding-bottom: 6px;
  transform-origin: bottom left;
  transform: scale(calc(1 / var(--canvas-zoom)));
}
.generator-mark {
  transform: scale(calc(1 / var(--canvas-zoom)));
}
</style>
