<script setup lang="ts">
import { clamp } from "es-toolkit";
import { computed, nextTick, onMounted, shallowRef, useTemplateRef } from "vue";

import type { WorldBounds } from "../../../model/structure/world-bounds";
import type { CanvasCamera } from "../viewport/canvas-camera";
import { useCameraStyle } from "../viewport/use-camera-style";

const props = defineProps<{
  bounds: WorldBounds;
  camera: CanvasCamera;
  error: string | undefined;
  title: string;
  zoom: number;
}>();
const emit = defineEmits<{ cancel: []; save: [title: string] }>();
const input = useTemplateRef<HTMLInputElement>("input");
const overlay = useTemplateRef<HTMLElement>("overlay");
const draft = shallowRef(props.title);
const width = computed(
  () => `${clamp((props.bounds.right - props.bounds.left) * props.zoom - 48, 120, 320)}px`,
);
useCameraStyle(props.camera, overlay, ({ x, y, zoom }) => ({
  left: `${x + props.bounds.left * zoom + 8}px`,
  top: `${y + props.bounds.top * zoom + 7}px`,
}));

function save() {
  const title = draft.value.trim();
  if (title) emit("save", title);
  else emit("cancel");
}

function keydown(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (event.key === "Enter") save();
  else if (event.key === "Escape") emit("cancel");
  else return;
  event.preventDefault();
  event.stopPropagation();
}

onMounted(() => void nextTick(() => input.value?.select()));
</script>

<template>
  <div ref="overlay" class="absolute z-40" :style="{ width }">
    <input
      ref="input"
      v-model="draft"
      class="h-7 w-full rounded-md border border-stroke-accent bg-surface-muted px-2 text-body text-fg-primary shadow-popover outline-none"
      :aria-label="title"
      :aria-invalid="Boolean(error)"
      @blur="emit('cancel')"
      @keydown="keydown"
    />
    <p
      v-if="error"
      class="mt-1 rounded-md bg-surface-danger px-2 py-1 text-caption text-fg-on-danger shadow-popover"
    >
      {{ error }}
    </p>
  </div>
</template>
