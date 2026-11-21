<script setup lang="ts">
import { computed } from "vue";

type ScrollFadeSurface = "canvas" | "raised" | "panel" | "muted";

const TOP_SURFACE_CLASS: Record<ScrollFadeSurface, string> = {
  canvas: "bg-linear-to-b from-surface-canvas to-transparent",
  raised: "bg-linear-to-b from-surface-raised to-transparent",
  panel: "bg-linear-to-b from-surface-panel to-transparent",
  muted: "bg-linear-to-b from-surface-muted to-transparent",
};

const BOTTOM_SURFACE_CLASS: Record<ScrollFadeSurface, string> = {
  canvas: "bg-linear-to-t from-surface-canvas to-transparent",
  raised: "bg-linear-to-t from-surface-raised to-transparent",
  panel: "bg-linear-to-t from-surface-panel to-transparent",
  muted: "bg-linear-to-t from-surface-muted to-transparent",
};

const props = withDefaults(
  defineProps<{
    surface?: ScrollFadeSurface;
    edges?: "both" | "top" | "bottom";
    topClass?: string;
    bottomClass?: string;
  }>(),
  {
    surface: "raised",
    edges: "both",
    topClass: "h-7",
    bottomClass: "h-10",
  },
);

const showTop = computed(() => props.edges === "both" || props.edges === "top");
const showBottom = computed(() => props.edges === "both" || props.edges === "bottom");
</script>

<template>
  <div
    v-if="showTop"
    aria-hidden="true"
    class="pointer-events-none absolute inset-x-0 top-0 z-5"
    :class="[topClass, TOP_SURFACE_CLASS[surface]]"
  />
  <div
    v-if="showBottom"
    aria-hidden="true"
    class="pointer-events-none absolute inset-x-0 bottom-0 z-5"
    :class="[bottomClass, BOTTOM_SURFACE_CLASS[surface]]"
  />
</template>
