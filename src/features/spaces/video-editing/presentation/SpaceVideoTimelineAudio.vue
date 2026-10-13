<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VideoAudioClip } from "../project";
import { stepAudioTrimFrame } from "../timeline/frame-step";
import { timeToPx } from "../timeline/scale";
import { clipLengthMs } from "../timing";
import SpaceVideoWaveform from "./SpaceVideoWaveform.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ clip: VideoAudioClip; editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, selection, materials, visibleProject, commit } = props.editor;
const EDGES = ["start", "end"] as const;

const selected = computed(() => selection.selection.value?.id === props.clip.id);
const geometry = computed(() => ({
  left: `${timeToPx(props.clip.timelineStartMs, timeline.pxPerSecond.value)}px`,
  width: `${Math.max(10, timeToPx(clipLengthMs(props.clip), timeline.pxPerSecond.value))}px`,
}));
const title = computed(
  () =>
    materials.available.value.get(props.clip.assetId)?.data.title ??
    t("spaces.media.videoEditor.materials.unavailable"),
);

function grab(event: PointerEvent) {
  gestures.begin(event, {
    kind: "audio-move",
    id: props.clip.id,
    grabMs: timeline.timeAt(event) - props.clip.timelineStartMs,
  });
}

function nudge(edge: "start" | "end", direction: -1 | 1) {
  const project = visibleProject.value;
  if (!project) return;
  const next = stepAudioTrimFrame(project, props.clip, edge, direction);
  if (next) commit(next, `audio-trim:${props.clip.id}`);
}
</script>

<template>
  <div
    class="absolute inset-y-0.5 overflow-hidden rounded border bg-surface-success-subtle"
    :class="selected ? 'border-stroke-accent ring-1 ring-stroke-accent' : 'border-stroke-default'"
    :style="geometry"
  >
    <SpaceVideoWaveform :clip="clip" :editor="editor" />
    <button
      type="button"
      class="absolute inset-0 w-full cursor-grab touch-none truncate px-2 text-left text-caption-sm text-fg-secondary focus-visible:outline-2 focus-visible:outline-stroke-focus active:cursor-grabbing"
      :aria-label="title"
      @pointerdown="grab"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    >
      {{ title }}
    </button>
    <button
      v-for="edge in EDGES"
      :key="edge"
      type="button"
      class="absolute inset-y-0 w-1.5 cursor-ew-resize touch-none bg-surface-accent/40 hover:bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="edge === 'start' ? 'left-0' : 'right-0'"
      :aria-label="
        t(edge === 'start' ? 'spaces.media.videoEditor.trim.start' : 'spaces.media.videoEditor.trim.end')
      "
      @pointerdown="gestures.begin($event, { kind: 'audio-trim', id: clip.id, edge })"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
      @keydown.left.prevent="nudge(edge, -1)"
      @keydown.right.prevent="nudge(edge, 1)"
    />
  </div>
</template>
