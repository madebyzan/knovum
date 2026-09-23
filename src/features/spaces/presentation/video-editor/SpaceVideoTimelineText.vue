<script setup lang="ts">
import { computed, shallowRef } from "vue";

import type { VideoTextOverlay } from "../../video-editing/project";

const props = defineProps<{
  item: VideoTextOverlay;
  durationMs: number;
  selected: boolean;
}>();
const emit = defineEmits<{
  select: [];
  edit: [item: VideoTextOverlay];
}>();

type GestureKind = "move" | "start" | "end";
const edges = ["start", "end"] as const;
const gesture = shallowRef<{ kind: GestureKind; originX: number; trackWidth: number }>();
const draft = shallowRef<VideoTextOverlay>();
const visible = computed(() => draft.value ?? props.item);
const position = computed(() => ({
  left: `${(visible.value.startMs / props.durationMs) * 100}%`,
  width: `${((visible.value.endMs - visible.value.startMs) / props.durationMs) * 100}%`,
}));

function change(kind: GestureKind, delta: number) {
  const original = props.item;
  if (kind === "move") {
    const length = original.endMs - original.startMs;
    const startMs = Math.max(0, Math.min(props.durationMs - length, original.startMs + delta));
    return { ...original, startMs, endMs: startMs + length };
  }
  if (kind === "start")
    return { ...original, startMs: Math.max(0, Math.min(original.endMs - 100, original.startMs + delta)) };
  return {
    ...original,
    endMs: Math.max(original.startMs + 100, Math.min(props.durationMs, original.endMs + delta)),
  };
}

function start(event: PointerEvent, kind: GestureKind) {
  if (event.button !== 0) return;
  const trackWidth = (event.currentTarget as HTMLElement)
    .closest<HTMLElement>("[data-text-track]")
    ?.getBoundingClientRect().width;
  if (!trackWidth) return;
  event.stopPropagation();
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  gesture.value = { kind, originX: event.clientX, trackWidth };
  draft.value = props.item;
  emit("select");
}

function move(event: PointerEvent) {
  const current = gesture.value;
  if (!current) return;
  const rawDelta = ((event.clientX - current.originX) / current.trackWidth) * props.durationMs;
  const delta = event.altKey ? Math.round(rawDelta) : Math.round(rawDelta / 100) * 100;
  draft.value = change(current.kind, delta);
}

function finish() {
  const next = draft.value;
  gesture.value = undefined;
  draft.value = undefined;
  if (next && (next.startMs !== props.item.startMs || next.endMs !== props.item.endMs)) emit("edit", next);
}

function cancel() {
  gesture.value = undefined;
  draft.value = undefined;
}

function nudge(kind: GestureKind, delta: number) {
  const next = change(kind, delta);
  if (next.startMs !== props.item.startMs || next.endMs !== props.item.endMs) emit("edit", next);
}
</script>

<template>
  <div class="absolute top-1 h-8 min-w-0" :style="position">
    <button
      type="button"
      class="size-full cursor-grab touch-none overflow-hidden rounded-grid-cell border bg-selected px-3 text-left text-caption text-accent focus-visible:outline-2 focus-visible:outline-focus-ring active:cursor-grabbing"
      :class="selected ? 'border-accent ring-1 ring-accent' : 'border-accent/25'"
      :aria-label="item.text"
      @click="$emit('select')"
      @pointerdown="start($event, 'move')"
      @pointermove="move"
      @pointerup="finish"
      @pointercancel="cancel"
      @keydown.left.prevent="nudge('move', -100)"
      @keydown.right.prevent="nudge('move', 100)"
    >
      <span class="block truncate">{{ item.text }}</span>
    </button>
    <button
      v-for="edge in edges"
      :key="edge"
      type="button"
      class="absolute inset-y-0 w-2 cursor-ew-resize touch-none bg-accent/25 hover:bg-accent/50 focus-visible:outline-2 focus-visible:outline-focus-ring"
      :class="edge === 'start' ? 'left-0 rounded-l-grid-cell' : 'right-0 rounded-r-grid-cell'"
      :aria-label="$t(edge === 'start' ? 'spaces.media.trimTextStart' : 'spaces.media.trimTextEnd')"
      @pointerdown="start($event, edge)"
      @pointermove="move"
      @pointerup="finish"
      @pointercancel="cancel"
      @keydown.left.prevent="nudge(edge, -100)"
      @keydown.right.prevent="nudge(edge, 100)"
    />
  </div>
</template>
