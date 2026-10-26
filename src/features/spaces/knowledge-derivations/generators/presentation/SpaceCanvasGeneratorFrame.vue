<script setup lang="ts">
import { FileText, Image, LoaderCircle, Play } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getCanvasEditorSession } from "../../../canvas/editor-session";
import { generatorFrame, type CanvasGenerator } from "../canvas-generator";
import { useCanvasGeneratorStore } from "../generator-store";

/**
 * Where an output will land, drawn at the size it will take. It behaves like the canvas objects
 * around it: pressing selects it, dragging moves it, Delete removes it and Esc leaves it. Its
 * label and border stay screen-sized so it reads the same at any zoom.
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
  if (event.button !== 0) return;
  const element = event.currentTarget as HTMLElement;
  // Selecting the generator puts the canvas selection down, as selecting a node replaces it.
  editorSession.selectNodes([]);
  store.activate(props.generator.id);
  element.focus({ preventScroll: true });
  if (working.value) return;
  drag = {
    pointerId: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    zoom: editorSession.viewport.value?.zoom ?? 1,
  };
  element.setPointerCapture(event.pointerId);
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

function remove(event: KeyboardEvent) {
  if (working.value) return;
  event.preventDefault();
  store.discard(props.generator.id);
}
</script>

<template>
  <div
    data-canvas-generator
    class="pointer-events-auto absolute touch-none outline-none"
    :class="working ? 'cursor-default' : 'cursor-grab active:cursor-grabbing'"
    :style="{
      left: `${frame.x}px`,
      top: `${frame.y}px`,
      width: `${frame.width}px`,
      height: `${frame.height}px`,
    }"
    role="group"
    tabindex="-1"
    :aria-label="t(`spaces.creation.generator.kinds.${generator.kind}`)"
    @pointerdown="press"
    @pointermove="follow"
    @pointerup="release"
    @pointercancel="release"
    @keydown.delete="remove"
    @keydown.escape="store.deactivate()"
    @contextmenu.prevent
  >
    <div
      class="generator-label flex items-center gap-1.5 text-caption"
      :class="active ? 'text-fg-accent' : 'text-fg-secondary'"
    >
      <component :is="ICONS[generator.kind]" class="size-3.5 shrink-0" :stroke-width="1.8" />
      <span class="min-w-0 truncate font-medium">{{
        t(`spaces.creation.generator.kinds.${generator.kind}`)
      }}</span>
      <span v-if="working" class="ml-auto flex shrink-0 items-center gap-1 text-fg-tertiary">
        <LoaderCircle class="size-3 animate-spin" :stroke-width="2" />
        {{ t("spaces.creation.generator.generating") }}
      </span>
      <span v-else-if="detail" class="ml-auto shrink-0 text-fg-tertiary tabular-nums">{{ detail }}</span>
    </div>
    <div
      class="generator-frame grid size-full place-items-center bg-surface-muted"
      :class="working && 'animate-pulse'"
      :data-active="active || undefined"
    >
      <component
        :is="ICONS[generator.kind]"
        class="generator-mark size-8 text-fg-tertiary opacity-60"
        :stroke-width="1.25"
      />
    </div>
  </div>
</template>

<style scoped>
/* `--canvas-zoom` is published by the camera on the world layer: what is divided by it stays screen-sized. */
.generator-frame {
  border: calc(1px / var(--canvas-zoom)) solid var(--color-stroke-default);
  border-radius: calc(12px / var(--canvas-zoom));
  transition: border-color 120ms ease;
}
.generator-frame:hover {
  border-color: var(--color-stroke-strong);
}
.generator-frame[data-active] {
  border-color: var(--color-stroke-accent);
  box-shadow: 0 0 0 calc(1px / var(--canvas-zoom)) var(--color-stroke-accent);
}
.generator-label {
  position: absolute;
  bottom: 100%;
  left: 0;
  width: calc(100% * var(--canvas-zoom));
  padding: 0 2px 6px;
  transform-origin: bottom left;
  transform: scale(calc(1 / var(--canvas-zoom)));
}
.generator-mark {
  transform: scale(calc(1 / var(--canvas-zoom)));
}
</style>
