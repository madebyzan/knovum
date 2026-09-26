<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, inject, onMounted, onUnmounted, useTemplateRef } from "vue";

import { canvasContentMeasurementKey } from "../canvas-content-measurement";

const props = withDefaults(defineProps<{ measure?: boolean; scale: number }>(), { measure: true });
const emit = defineEmits<{ contentHeight: [height: number] }>();
const plane = useTemplateRef<HTMLElement>("plane");
const measurements = inject(canvasContentMeasurementKey);
const style = computed(() => ({
  width: `${100 / props.scale}%`,
  transform: `scale(${props.scale})`,
}));

let detachMeasurement: (() => void) | undefined;
function reportContentHeight() {
  const element = plane.value;
  if (element) emit("contentHeight", element.scrollHeight * props.scale);
}
useResizeObserver(() => (props.measure ? plane.value : null), reportContentHeight);
onMounted(() => {
  if (props.measure) detachMeasurement = measurements?.register(reportContentHeight);
});
onUnmounted(() => {
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
