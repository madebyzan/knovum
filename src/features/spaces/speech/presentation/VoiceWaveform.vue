<script setup lang="ts">
import { useRafFn } from "@vueuse/core";
import { ref, watch } from "vue";

const props = defineProps<{
  /** Reads the loudest audible sample, from 0 to 1. */
  peak: () => number;
  active: boolean;
  bars?: number;
}>();

const BARS = props.bars ?? 28;
const levels = ref<number[]>(Array.from({ length: BARS }, () => 0));

// The bars scroll with the voice: each frame appends what is audible now and drops the oldest, so
// the shape a person sees is the sound they are hearing.
const { pause, resume } = useRafFn(
  () => {
    levels.value = [...levels.value.slice(1), props.peak()];
  },
  { immediate: false },
);

watch(
  () => props.active,
  (active) => {
    if (active) {
      resume();
      return;
    }
    pause();
    levels.value = Array.from({ length: BARS }, () => 0);
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex h-5 items-center gap-px" aria-hidden="true">
    <span
      v-for="(level, index) in levels"
      :key="index"
      class="w-0.5 rounded-full bg-fg-tertiary transition-[height] duration-75"
      :class="active ? 'bg-fg-accent' : 'bg-stroke-divider'"
      :style="{ height: `${Math.max(2, Math.round(level * 20))}px` }"
    />
  </div>
</template>
