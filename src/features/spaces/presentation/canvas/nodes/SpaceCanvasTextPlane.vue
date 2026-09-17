<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, useTemplateRef } from "vue";

import { canvasContentMeasurementKey } from "../canvas-content-measurement";

const props = defineProps<{ scale: number }>();
const emit = defineEmits<{ contentHeight: [height: number] }>();
const plane = useTemplateRef<HTMLElement>("plane");
const measurements = inject(canvasContentMeasurementKey);
const style = computed(() => ({
  width: `${100 / props.scale}%`,
  transform: `scale(${props.scale})`,
}));

let observer: ResizeObserver | undefined;
let detachMeasurement: (() => void) | undefined;
function measure() {
  const element = plane.value;
  if (element) emit("contentHeight", element.scrollHeight * props.scale);
}
onMounted(() => {
  const element = plane.value;
  if (!element) return;
  observer = new ResizeObserver(measure);
  observer.observe(element);
  detachMeasurement = measurements?.register(measure);
});
onUnmounted(() => {
  observer?.disconnect();
  detachMeasurement?.();
});
</script>

<template>
  <div
    ref="plane"
    class="canvas-text-prose w-full origin-top-left self-start overflow-hidden px-2 py-1 leading-tight wrap-break-word"
    :style="style"
  >
    <slot />
  </div>
</template>
