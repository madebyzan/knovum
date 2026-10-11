<script setup lang="ts">
import { computed, type CSSProperties } from "vue";

const props = defineProps<{
  position?: { x: number; y: number };
  rect: { width: number; height: number };
  diameter: number;
}>();
const style = computed<CSSProperties>(() => {
  const brush = Math.min(props.rect.width, props.rect.height) * props.diameter;
  return {
    left: `${props.position?.x ?? 0}px`,
    top: `${props.position?.y ?? 0}px`,
    width: `${brush}px`,
    height: `${brush}px`,
  };
});
</script>

<template>
  <span
    v-if="position"
    class="pointer-events-none absolute z-30 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-stroke-on-image bg-surface-raised/20 ring-1 ring-black/70"
    :style="style"
  />
</template>
