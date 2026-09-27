<script setup lang="ts">
import { clamp } from "es-toolkit";
import { nextTick, onMounted, shallowRef, useTemplateRef } from "vue";

import type { WorldBounds } from "../../../model/structure/world-bounds";
import type { CanvasViewport } from "../../contracts/canvas-editor-port";

const props = defineProps<{
  bounds: WorldBounds;
  error: string | undefined;
  title: string;
  viewport: CanvasViewport;
}>();
const emit = defineEmits<{ cancel: []; save: [title: string] }>();
const input = useTemplateRef<HTMLInputElement>("input");
const draft = shallowRef(props.title);

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
  <div
    class="absolute z-40"
    :style="{
      left: `${viewport.x + bounds.left * viewport.zoom + 8}px`,
      top: `${viewport.y + bounds.top * viewport.zoom + 7}px`,
      width: `${clamp((bounds.right - bounds.left) * viewport.zoom - 48, 120, 320)}px`,
    }"
  >
    <input
      ref="input"
      v-model="draft"
      class="h-7 w-full rounded-control border border-accent bg-surface px-2 text-body text-foreground shadow-popover outline-none"
      :aria-label="title"
      :aria-invalid="Boolean(error)"
      @blur="emit('cancel')"
      @keydown="keydown"
    />
    <p
      v-if="error"
      class="mt-1 rounded-control bg-danger px-2 py-1 text-caption text-on-danger shadow-popover"
    >
      {{ error }}
    </p>
  </div>
</template>
