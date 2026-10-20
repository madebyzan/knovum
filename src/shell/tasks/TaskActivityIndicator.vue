<script setup lang="ts">
import { computed } from "vue";

export type TaskIndicatorVariant = "waveform" | "orbit" | "pulse" | "ring";

const props = withDefaults(
  defineProps<{
    variant?: TaskIndicatorVariant;
    progress?: number | undefined;
  }>(),
  {
    variant: "waveform",
    progress: undefined,
  },
);

const PROGRESS_RING_CIRCUMFERENCE = 37.7;
const strokeDashoffset = computed(() => {
  if (props.progress === undefined) return undefined;
  const clamped = Math.max(0, Math.min(100, props.progress));
  return PROGRESS_RING_CIRCUMFERENCE * (1 - clamped / 100);
});
</script>

<template>
  <!-- 1. Waveform: Audio / AI Computing activity waveform (3 bars, zero rotation, no jitter) -->
  <span
    v-if="variant === 'waveform'"
    class="inline-flex h-3.5 w-3.5 shrink-0 items-center justify-center gap-[2px] text-fg-accent"
    aria-hidden="true"
  >
    <span class="activity-bar activity-bar-1" />
    <span class="activity-bar activity-bar-2" />
    <span class="activity-bar activity-bar-3" />
  </span>

  <!-- 2. Orbit: Knowledge nodes orbiting in dynamic equilibrium -->
  <svg
    v-else-if="variant === 'orbit'"
    class="size-3.5 shrink-0 text-fg-accent"
    viewBox="0 0 16 16"
    fill="none"
    aria-hidden="true"
  >
    <circle cx="8" cy="8" r="5.5" stroke="currentColor" stroke-width="1.2" class="opacity-20" />
    <g class="orbit-spinner">
      <circle cx="8" cy="2.5" r="1.5" fill="currentColor" />
      <circle cx="8" cy="13.5" r="1.1" fill="currentColor" class="opacity-70" />
    </g>
  </svg>

  <!-- 3. Pulse: Apple-like breathing status beacon -->
  <span
    v-else-if="variant === 'pulse'"
    class="relative inline-flex size-3.5 shrink-0 items-center justify-center text-fg-accent"
    aria-hidden="true"
  >
    <span class="beacon-halo" />
    <span class="relative size-1.5 rounded-full bg-current" />
  </span>

  <!-- 4. Ring: Precision anchored circular progress or sweep -->
  <svg
    v-else
    class="size-3.5 shrink-0 text-fg-accent"
    :class="{ 'animate-spin motion-reduce:animate-none': progress === undefined }"
    viewBox="0 0 16 16"
    fill="none"
    aria-hidden="true"
    style="transform-origin: center; will-change: transform"
  >
    <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.8" class="opacity-20" />
    <circle
      cx="8"
      cy="8"
      r="6"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      :stroke-dasharray="PROGRESS_RING_CIRCUMFERENCE"
      :stroke-dashoffset="strokeDashoffset ?? 12"
      :class="{ 'origin-center -rotate-90 transition-all duration-300': progress !== undefined }"
    />
  </svg>
</template>

<style scoped>
.activity-bar {
  width: 2px;
  border-radius: 9999px;
  background-color: currentColor;
  transform-origin: center;
  will-change: transform;
}
.activity-bar-1 {
  height: 8px;
  animation: task-wave 1.2s ease-in-out infinite alternate;
}
.activity-bar-2 {
  height: 12px;
  animation: task-wave 1.2s ease-in-out infinite alternate 0.4s;
}
.activity-bar-3 {
  height: 6px;
  animation: task-wave 1.2s ease-in-out infinite alternate 0.2s;
}

@keyframes task-wave {
  0% {
    transform: scaleY(0.35);
  }
  100% {
    transform: scaleY(1);
  }
}

.orbit-spinner {
  transform-origin: 8px 8px;
  animation: task-orbit 1.8s linear infinite;
  will-change: transform;
}

@keyframes task-orbit {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.beacon-halo {
  position: absolute;
  inset: 0.5px;
  border-radius: 9999px;
  background-color: currentColor;
  opacity: 0.25;
  animation: task-beacon 2s cubic-bezier(0.2, 0.8, 0.2, 1) infinite;
  will-change: transform, opacity;
}

@keyframes task-beacon {
  0% {
    transform: scale(0.6);
    opacity: 0.6;
  }
  100% {
    transform: scale(1.6);
    opacity: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .activity-bar,
  .orbit-spinner,
  .beacon-halo {
    animation: none;
  }
}
</style>
