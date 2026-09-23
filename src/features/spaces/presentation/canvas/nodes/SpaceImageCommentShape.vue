<script setup lang="ts">
import { computed } from "vue";

import type { CommentMark } from "../../../comments/use-image-comment-gesture";

const props = defineProps<{ mark: CommentMark }>();
const points = computed(() => {
  const shape = props.mark.shape;
  if (shape?.kind !== "stroke") return "";
  return Array.from(
    { length: shape.points.length / 2 },
    (_, index) => `${shape.points[index * 2]! * 1000},${shape.points[index * 2 + 1]! * 1000}`,
  ).join(" ");
});
</script>

<template>
  <svg
    v-if="mark.shape"
    class="pointer-events-none absolute inset-0 size-full overflow-visible text-canvas-relation-purple"
    viewBox="0 0 1000 1000"
    preserveAspectRatio="none"
    aria-hidden="true"
  >
    <polyline
      v-if="mark.shape.kind === 'stroke'"
      :points="points"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
    <g v-else>
      <rect
        :x="mark.shape.x * 1000"
        :y="mark.shape.y * 1000"
        :width="mark.shape.width * 1000"
        :height="mark.shape.height * 1000"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-dasharray="6 4"
        vector-effect="non-scaling-stroke"
      />
      <circle :cx="mark.shape.x * 1000" :cy="mark.shape.y * 1000" r="9" fill="currentColor" />
      <circle
        :cx="(mark.shape.x + mark.shape.width) * 1000"
        :cy="(mark.shape.y + mark.shape.height) * 1000"
        r="9"
        fill="currentColor"
      />
    </g>
  </svg>
</template>
