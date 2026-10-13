<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VideoClip } from "../project";
import { stepTrimFrame } from "../timeline/frame-step";
import { timeToPx } from "../timeline/scale";
import { clipLengthMs } from "../timing";
import SpaceVideoFilmstrip from "./SpaceVideoFilmstrip.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ clip: VideoClip; editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, selection, visibleProject, materials, commit } = props.editor;
const EDGES = ["start", "end"] as const;

const selected = computed(() => selection.selection.value?.id === props.clip.id);
const geometry = computed(() => ({
  left: `${timeToPx(props.clip.timelineStartMs, timeline.pxPerSecond.value)}px`,
  width: `${Math.max(8, timeToPx(clipLengthMs(props.clip), timeline.pxPerSecond.value))}px`,
}));
const title = computed(
  () =>
    materials.available.value.get(props.clip.assetId)?.data.title ??
    t("spaces.media.videoEditor.materials.unavailable"),
);

function grab(event: PointerEvent) {
  gestures.begin(event, {
    kind: "clip-move",
    id: props.clip.id,
    grabMs: timeline.timeAt(event) - props.clip.timelineStartMs,
  });
}

function nudge(edge: "start" | "end", direction: -1 | 1) {
  const project = visibleProject.value;
  if (!project) return;
  const next = stepTrimFrame(project, props.clip, edge, direction);
  if (next) commit(next, `clip-trim:${props.clip.id}`);
}
</script>

<template>
  <div
    class="absolute inset-y-1 overflow-hidden rounded-md border bg-surface-accent-subtle select-none"
    :class="selected ? 'border-stroke-accent ring-1 ring-stroke-accent' : 'border-stroke-accent/30'"
    :style="geometry"
    :data-clip-id="clip.id"
  >
    <SpaceVideoFilmstrip :clip="clip" :editor="editor" />
    <button
      type="button"
      class="absolute inset-0 w-full cursor-grab touch-none px-2 text-left focus-visible:outline-2 focus-visible:outline-stroke-focus active:cursor-grabbing"
      :aria-label="`${title} · ${t('spaces.media.videoEditor.clip.selected')}`"
      @pointerdown="grab"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    >
      <span
        class="absolute top-1 left-1 max-w-[calc(100%-0.5rem)] truncate rounded bg-black/45 px-1 text-caption-sm text-fg-on-image"
        >{{ title }}</span
      >
    </button>
    <button
      v-for="edge in EDGES"
      :key="edge"
      type="button"
      class="absolute inset-y-0 z-10 w-2 cursor-ew-resize touch-none bg-surface-accent/60 hover:bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="edge === 'start' ? 'left-0' : 'right-0'"
      :aria-label="
        t(edge === 'start' ? 'spaces.media.videoEditor.trim.start' : 'spaces.media.videoEditor.trim.end')
      "
      @pointerdown="gestures.begin($event, { kind: 'clip-trim', id: clip.id, edge })"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
      @keydown.left.prevent="nudge(edge, -1)"
      @keydown.right.prevent="nudge(edge, 1)"
    />
    <span
      v-if="clip.fadeInMs"
      class="pointer-events-none absolute top-0 left-0 h-full bg-gradient-to-r from-black/55 to-transparent"
      :style="{ width: `${timeToPx(clip.fadeInMs, timeline.pxPerSecond.value)}px` }"
    />
    <span
      v-if="clip.fadeOutMs"
      class="pointer-events-none absolute top-0 right-0 h-full bg-gradient-to-l from-black/55 to-transparent"
      :style="{ width: `${timeToPx(clip.fadeOutMs, timeline.pxPerSecond.value)}px` }"
    />
  </div>
</template>
