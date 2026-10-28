<script setup lang="ts">
import { computed } from "vue";

/** A frame's proportions drawn as an outlined rectangle inside a square icon box. */
const props = defineProps<{ width: number; height: number }>();
const shape = computed(() => {
  const ratio = props.width / props.height;
  // The longer side fills the box; the shorter keeps a readable minimum so extreme ratios stay legible.
  const short = `${Math.max(35, Math.round(100 * Math.min(ratio, 1 / ratio)))}%`;
  return ratio >= 1 ? { width: "100%", height: short } : { width: short, height: "100%" };
});
</script>

<template>
  <span class="grid size-3.5 shrink-0 place-items-center" aria-hidden="true">
    <span class="rounded-[2px] border-[1.5px] border-current" :style="shape" />
  </span>
</template>
