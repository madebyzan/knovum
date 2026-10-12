<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";

import type { VideoMaterialNode } from "../material";
import { previewFrames, previewTimeMs } from "../preview-composition";
import type { VideoEditProject } from "../project";
import SpaceVideoPreviewAudio from "./SpaceVideoPreviewAudio.vue";
import SpaceVideoPreviewClip from "./SpaceVideoPreviewClip.vue";
import SpaceVideoPreviewText from "./SpaceVideoPreviewText.vue";

const props = defineProps<{
  project: VideoEditProject;
  materials: ReadonlyMap<string, VideoMaterialNode>;
  spaceId: string;
  timelineMs: number;
  playing: boolean;
  selectedTextId?: string | undefined;
}>();
const emit = defineEmits<{ failed: [error: unknown]; ready: [value: boolean]; selectText: [id: string] }>();
const viewport = shallowRef<HTMLElement | null>(null);
const bounds = shallowRef({ width: 0, height: 0 });
const scale = computed(() =>
  Math.min(
    bounds.value.width / props.project.output.width,
    bounds.value.height / props.project.output.height,
  ),
);
const readiness = shallowRef<ReadonlyMap<string, boolean>>(new Map());
function ready(id: string, value: boolean) {
  if (readiness.value.get(id) !== value) readiness.value = new Map(readiness.value).set(id, value);
}
const frames = computed(() =>
  previewFrames(props.project, props.timelineMs).map((frame) => ({
    frame,
    asset: props.project.assets.find((asset) => asset.nodeId === frame.clip.assetId)!,
    node: props.materials.get(frame.clip.assetId),
  })),
);
const sampleMs = computed(() => previewTimeMs(props.project, props.timelineMs));
const audio = computed(() =>
  props.project.audioTrack.clips
    .filter(
      (clip) =>
        sampleMs.value >= clip.timelineStartMs &&
        sampleMs.value < clip.timelineStartMs + clip.endMs - clip.startMs,
    )
    .map((clip) => ({ clip, node: props.materials.get(clip.assetId) })),
);
watch([frames, audio], () => {
  const activeIds = new Set([
    ...frames.value.map(({ frame }) => frame.clip.id),
    ...audio.value.map(({ clip }) => clip.id),
  ]);
  const retained = [...readiness.value].filter(([id]) => activeIds.has(id));
  if (retained.length !== readiness.value.size) readiness.value = new Map(retained);
});
const unavailable = computed(
  () => frames.value.some(({ node }) => !node) || audio.value.some(({ node }) => !node),
);
useResizeObserver(viewport, ([entry]) => {
  if (entry) bounds.value = { width: entry.contentRect.width, height: entry.contentRect.height };
});
const playable = computed(
  () =>
    !unavailable.value &&
    frames.value.every(({ frame }) => readiness.value.get(frame.clip.id)) &&
    audio.value.every(({ clip }) => readiness.value.get(clip.id)),
);
watch(playable, (value) => emit("ready", value), { immediate: true });
</script>
<template>
  <div
    :ref="(value) => (viewport = value as HTMLElement | null)"
    class="relative grid min-h-0 min-w-0 flex-1 place-items-center overflow-hidden bg-surface-muted p-4"
  >
    <p v-if="unavailable" role="alert" class="px-4 text-center text-caption text-fg-danger">
      {{ $t("spaces.media.videoEditor.materials.unavailable") }}
    </p>
    <div
      v-else
      class="relative isolate overflow-hidden bg-black"
      :style="{ width: `${project.output.width * scale}px`, height: `${project.output.height * scale}px` }"
    >
      <template v-for="entry in frames" :key="entry.frame.clip.id">
        <SpaceVideoPreviewClip
          v-if="entry.node"
          :frame="entry.frame"
          :asset="entry.asset"
          :node="entry.node"
          :output="project.output"
          :scale="scale"
          :space-id="spaceId"
          :playing="playing && playable"
          @failed="emit('failed', $event)"
          @ready="ready(entry.frame.clip.id, $event)"
        />
      </template>
      <SpaceVideoPreviewText
        :project="project"
        :timeline-ms="sampleMs"
        :height="project.output.height * scale"
        :selected-text-id="selectedTextId"
        @select-text="emit('selectText', $event)"
      />
      <template v-for="entry in audio" :key="entry.clip.id">
        <SpaceVideoPreviewAudio
          v-if="entry.node"
          :clip="entry.clip"
          :node="entry.node"
          :timeline-ms="sampleMs"
          :playing="playing && playable"
          @failed="emit('failed', $event)"
          @ready="ready(entry.clip.id, $event)"
        />
      </template>
    </div>
  </div>
</template>
