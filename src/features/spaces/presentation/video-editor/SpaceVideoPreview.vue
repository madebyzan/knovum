<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, watch } from "vue";

import { type VideoEditProject, type VideoTextOverlay } from "../../video-editing/project";
import { versionedMediaSource } from "../media/element";

const props = defineProps<{
  sourcePath: string;
  project: VideoEditProject;
  timelineMs: number;
  selectedTextId?: string | undefined;
}>();
const emit = defineEmits<{ playing: [value: boolean]; ready: []; failed: []; selectText: [id: string] }>();
const video = defineModel<HTMLVideoElement | null>("element", { default: null });
const viewport = shallowRef<HTMLElement | null>(null);
const frame = shallowRef<{ width: number; height: number; videoWidth: number; videoHeight: number }>();
const source = computed(() => versionedMediaSource(props.sourcePath));
const overlays = computed(() =>
  props.project.textTrack.items.filter(
    ({ startMs, endMs }) => props.timelineMs >= startMs && props.timelineMs < endMs,
  ),
);
const rotation = computed(() => props.project.transform.rotation);
const frameStyle = computed(() =>
  frame.value ? { width: `${frame.value.width}px`, height: `${frame.value.height}px` } : undefined,
);
const videoStyle = computed(() => {
  if (!frame.value) return undefined;
  const value = props.project.transform;
  return {
    width: `${frame.value.videoWidth}px`,
    height: `${frame.value.videoHeight}px`,
    transform: `translate(-50%, -50%) scale(${value.flipHorizontal ? -1 : 1}, ${value.flipVertical ? -1 : 1}) rotate(${value.rotation}deg)`,
  };
});
function textStyle(overlay: VideoTextOverlay) {
  const style = overlay.style;
  const scale = (frame.value?.height ?? 0) / 720;
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

function updateGeometry() {
  const element = video.value;
  const bounds = viewport.value?.getBoundingClientRect();
  if (!element || !bounds || !hasGeometry(element, bounds)) return;
  const quarterTurn = rotation.value === 90 || rotation.value === 270;
  const orientedWidth = quarterTurn ? element.videoHeight : element.videoWidth;
  const orientedHeight = quarterTurn ? element.videoWidth : element.videoHeight;
  const scale = Math.min(bounds.width / orientedWidth, bounds.height / orientedHeight);
  frame.value = {
    width: orientedWidth * scale,
    height: orientedHeight * scale,
    videoWidth: element.videoWidth * scale,
    videoHeight: element.videoHeight * scale,
  };
}

function hasGeometry(element: HTMLVideoElement, bounds: DOMRect) {
  return element.videoWidth > 0 && element.videoHeight > 0 && bounds.width > 0 && bounds.height > 0;
}

function onReady() {
  updateGeometry();
  emit("ready");
}

let observer: ResizeObserver | undefined;
onMounted(() => {
  observer = new ResizeObserver(updateGeometry);
  if (viewport.value) observer.observe(viewport.value);
});
onBeforeUnmount(() => observer?.disconnect());
watch(rotation, updateGeometry);
</script>

<template>
  <div
    :ref="(value) => (viewport = value as HTMLElement | null)"
    class="relative grid min-h-0 flex-1 place-items-center overflow-hidden bg-control/25"
  >
    <div class="relative max-h-full max-w-full" :style="frameStyle">
      <video
        :ref="(value) => (video = value as HTMLVideoElement | null)"
        :src="source"
        class="absolute top-1/2 left-1/2 object-contain"
        :style="videoStyle"
        preload="metadata"
        @loadedmetadata="onReady"
        @error="$emit('failed')"
        @play="$emit('playing', true)"
        @pause="$emit('playing', false)"
      />
      <div
        v-for="overlay in frame ? overlays : []"
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
          class="pointer-events-auto max-w-full cursor-pointer font-normal whitespace-pre-line focus-visible:outline-2 focus-visible:outline-focus-ring"
          :class="selectedTextId === overlay.id && 'ring-1 ring-accent'"
          :style="textStyle(overlay)"
          :aria-label="overlay.text"
          @click="$emit('selectText', overlay.id)"
        >
          {{ overlay.text }}
        </button>
      </div>
    </div>
  </div>
</template>
