<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { timeToTimelineX } from "../timeline/scale";
import { rowOffsets, TIMELINE_ROWS } from "../timeline/track-geometry";
import SpaceVideoPlayhead from "./SpaceVideoPlayhead.vue";
import SpaceVideoTimelineAudio from "./SpaceVideoTimelineAudio.vue";
import SpaceVideoTimelineClip from "./SpaceVideoTimelineClip.vue";
import SpaceVideoTimelineRuler from "./SpaceVideoTimelineRuler.vue";
import SpaceVideoTimelineText from "./SpaceVideoTimelineText.vue";
import SpaceVideoTimelineTransition from "./SpaceVideoTimelineTransition.vue";
import SpaceVideoTrackHeaders from "./SpaceVideoTrackHeaders.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { editor } = props;
const { timeline, gestures, visibleProject, selection, player } = editor;
const headerScroller = useTemplateRef<HTMLElement>("headerScroller");
const project = computed(() => visibleProject.value!);
const rows = computed(() => rowOffsets(project.value));
const px = (timeMs: number) => timeToTimelineX(timeMs, timeline.pxPerSecond.value);
const endX = computed(() => px(timeline.durationMs.value));

function syncVerticalScroll(event: Event) {
  const target = event.currentTarget as HTMLElement | null;
  if (headerScroller.value && target) headerScroller.value.scrollTop = target.scrollTop;
}

function onEmptyTrackClick(event: MouseEvent) {
  if (event.target !== event.currentTarget) return;
  selection.clear();
  player.pause();
  player.seek(Math.min(timeline.timeAt(event), timeline.durationMs.value));
}
</script>

<template>
  <div class="flex min-h-0 flex-1 bg-surface-canvas" @keydown.esc.capture="gestures.cancel">
    <div ref="headerScroller" class="shrink-0 overflow-hidden bg-surface-panel">
      <SpaceVideoTrackHeaders :project="project" :editor="editor" />
    </div>
    <div
      :ref="(value) => (timeline.viewport.value = value as HTMLElement | null)"
      class="relative min-w-0 flex-1 overflow-x-auto overflow-y-auto"
      @wheel="timeline.onWheel"
      @scroll="syncVerticalScroll"
    >
      <div
        :ref="(value) => (timeline.surface.value = value as HTMLElement | null)"
        class="relative min-h-full"
        :style="{
          width: `${timeline.contentWidth.value}px`,
          minHeight: `${rows.audio + rows.lanes * TIMELINE_ROWS.audio}px`,
        }"
        :aria-label="t('spaces.media.player.timeline')"
        @click="onEmptyTrackClick"
      >
        <SpaceVideoTimelineRuler :editor="editor" />
        <div
          class="absolute inset-x-0 border-b border-stroke-divider bg-surface-panel/35"
          :style="{ top: `${rows.text}px`, height: `${TIMELINE_ROWS.text}px` }"
          @click="onEmptyTrackClick"
        >
          <SpaceVideoTimelineText
            v-for="item in project.textTrack.items"
            :key="item.id"
            :item="item"
            :editor="editor"
          />
        </div>
        <div
          class="absolute inset-x-0 border-b border-stroke-divider bg-surface-canvas"
          :style="{ top: `${rows.video}px`, height: `${TIMELINE_ROWS.video}px` }"
          @click="onEmptyTrackClick"
        >
          <SpaceVideoTimelineClip
            v-for="clip in project.videoTrack.clips"
            :key="clip.id"
            :clip="clip"
            :editor="editor"
          />
          <SpaceVideoTimelineTransition
            v-for="transition in project.transitions"
            :key="transition.id"
            :transition="transition"
            :editor="editor"
          />
        </div>
        <div
          v-for="lane in rows.lanes"
          :key="lane"
          class="absolute inset-x-0 border-b border-stroke-divider bg-surface-panel/35"
          :style="{ top: `${rows.lane(lane - 1)}px`, height: `${TIMELINE_ROWS.audio}px` }"
          @click="onEmptyTrackClick"
        >
          <SpaceVideoTimelineAudio
            v-for="clip in project.audioTrack.clips.filter((entry) => entry.lane === lane - 1)"
            :key="clip.id"
            :clip="clip"
            :editor="editor"
          />
        </div>
        <div
          v-if="endX < timeline.contentWidth.value"
          class="pointer-events-none absolute inset-y-0 right-0 z-10 border-l border-dashed border-stroke-strong/70 bg-surface-muted/40"
          :style="{ left: `${endX}px`, top: `${TIMELINE_ROWS.ruler}px` }"
          aria-hidden="true"
        />
        <span
          v-if="gestures.snapMs.value !== undefined"
          class="pointer-events-none absolute inset-y-0 z-20 w-px bg-surface-accent"
          :style="{ transform: `translateX(${px(gestures.snapMs.value)}px)` }"
        />
        <SpaceVideoPlayhead :editor="editor" />
      </div>
    </div>
  </div>
</template>
