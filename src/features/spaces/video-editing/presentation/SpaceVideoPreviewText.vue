<script setup lang="ts">
import { computed } from "vue";

import type { VideoEditProject, VideoTextOverlay } from "../project";
const props = defineProps<{
  project: VideoEditProject;
  timelineMs: number;
  height: number;
  selectedTextId?: string | undefined;
}>();
defineEmits<{ selectText: [id: string] }>();
const overlays = computed(() =>
  props.project.textTrack.items.filter(
    ({ startMs, endMs }) => props.timelineMs >= startMs && props.timelineMs < endMs,
  ),
);
function textStyle(overlay: VideoTextOverlay) {
  const style = overlay.style;
  const scale = props.height / 720;
  const edge = Math.max(0.5, 2 * scale);
  const outline = `${edge}px ${edge}px 0 rgba(0,0,0,.75), -${edge}px ${edge}px 0 rgba(0,0,0,.75), ${edge}px -${edge}px 0 rgba(0,0,0,.75), -${edge}px -${edge}px 0 rgba(0,0,0,.75)`;
  return {
    fontFamily: style.family,
    fontWeight: style.weight,
    color: style.color,
    fontSize: `${Math.max(1, style.size * scale)}px`,
    textShadow:
      style.preset === "outline"
        ? outline
        : style.preset === "shadow"
          ? `${edge}px ${edge}px ${edge * 2}px rgba(0,0,0,.8)`
          : undefined,
    backgroundColor: style.preset === "badge" ? "rgba(0,0,0,.55)" : undefined,
    padding: style.preset === "badge" ? `${8 * scale}px` : undefined,
  };
}
</script>
<template>
  <div
    v-for="overlay in overlays"
    :key="overlay.id"
    class="pointer-events-none absolute inset-x-[8%] flex justify-center text-center"
    :class="
      overlay.placement === 'top'
        ? 'top-[8%]'
        : overlay.placement === 'center'
          ? 'top-1/2 -translate-y-1/2'
          : 'bottom-[8%]'
    "
  >
    <button
      type="button"
      class="pointer-events-auto max-w-full cursor-pointer font-normal whitespace-pre-line focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="selectedTextId === overlay.id && 'ring-1 ring-stroke-accent'"
      :style="textStyle(overlay)"
      :aria-label="overlay.text"
      @click="$emit('selectText', overlay.id)"
    >
      {{ overlay.text }}
    </button>
  </div>
</template>
