<script setup lang="ts">
import { useEventListener, usePreferredReducedMotion, useResizeObserver } from "@vueuse/core";
import { nextTick, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";

const props = defineProps<{ title: string }>();
const viewport = useTemplateRef<HTMLSpanElement>("viewport");
const text = useTemplateRef<HTMLSpanElement>("text");
const reducedMotion = usePreferredReducedMotion();
const scrolling = shallowRef(false);
const hovered = shallowRef(false);
let animation: Animation | undefined;
let measuredDistance: number | undefined;

function reset() {
  animation?.cancel();
  animation = undefined;
  measuredDistance = undefined;
  scrolling.value = false;
}

function play() {
  if (!hovered.value || reducedMotion.value === "reduce" || !viewport.value || !text.value) {
    reset();
    return;
  }
  const distance = text.value.scrollWidth - viewport.value.clientWidth;
  if (distance === measuredDistance) return;
  reset();
  if (distance <= 0) return;
  measuredDistance = distance;
  scrolling.value = true;
  const travel = (distance / 32) * 1_000;
  const end = `translateX(-${distance}px)`;
  animation = text.value.animate([{ transform: "translateX(0)" }, { transform: end }], {
    duration: travel,
    delay: 900,
    fill: "forwards",
    easing: "linear",
  });
}

function enter() {
  hovered.value = true;
  play();
}

function leave() {
  hovered.value = false;
  reset();
}

useResizeObserver(() => (hovered.value ? [viewport.value, text.value] : []), play);
useEventListener(() => viewport.value?.parentElement, "pointerenter", enter);
useEventListener(() => viewport.value?.parentElement, "pointerleave", leave);
watch(
  () => props.title,
  async () => {
    reset();
    await nextTick();
    play();
  },
);
watch(reducedMotion, play);
onBeforeUnmount(leave);
</script>

<template>
  <span
    ref="viewport"
    class="min-w-0 flex-1 overflow-hidden whitespace-nowrap"
    :class="{ 'text-ellipsis': !scrolling }"
    :title="title"
  >
    <span ref="text" class="inline-block whitespace-nowrap">{{ title }}</span>
  </span>
</template>
