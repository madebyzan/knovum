<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { timeToPx } from "../timeline/scale";
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
const { timeline, gestures, visibleProject, materials, engine, selection } = editor;
const project = computed(() => visibleProject.value!);
const rows = computed(() => rowOffsets(project.value));
const px = (timeMs: number) => timeToPx(timeMs, timeline.pxPerSecond.value);

function dropMaterial(event: DragEvent) {
  const id = event.dataTransfer?.getData("application/x-knovum-video-material");
  if (!id) return;
  event.preventDefault();
  const node = materials.nodes.value.find((candidate) => candidate.id === id);
  const surface = timeline.surface.value;
  if (!node || !surface) return;
  const atMs = Math.max(
    0,
    ((event.clientX - surface.getBoundingClientRect().left) * 1000) / timeline.pxPerSecond.value,
  );
  void materials.add(node, atMs);
}
</script>

<template>
  <div class="flex min-h-0 flex-1" @keydown.esc.capture="gestures.cancel">
    <SpaceVideoTrackHeaders :project="project" />
    <div
      :ref="(value) => (timeline.viewport.value = value as HTMLElement | null)"
      class="relative min-w-0 flex-1 overflow-x-auto overflow-y-auto"
      @wheel="timeline.onWheel"
    >
      <div
        :ref="(value) => (timeline.surface.value = value as HTMLElement | null)"
        class="relative"
        :style="{
          width: `${timeline.contentWidth.value}px`,
          height: `${rows.audio + rows.lanes * TIMELINE_ROWS.audio}px`,
        }"
        :aria-label="t('spaces.media.player.timeline')"
        @dragover.prevent
        @drop="dropMaterial"
      >
        <SpaceVideoTimelineRuler :editor="editor" />
        <div
          class="absolute inset-x-0 border-b border-stroke-divider bg-surface-panel/40"
          :style="{ top: `${rows.text}px`, height: `${TIMELINE_ROWS.text}px` }"
        >
          <SpaceVideoTimelineText
            v-for="item in project.textTrack.items"
            :key="item.id"
            :item="item"
            :editor="editor"
          />
        </div>
        <div
          class="absolute inset-x-0 border-b border-stroke-divider"
          :style="{ top: `${rows.video}px`, height: `${TIMELINE_ROWS.video}px` }"
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
          class="absolute inset-x-0 border-b border-stroke-divider"
          :style="{ top: `${rows.lane(lane - 1)}px`, height: `${TIMELINE_ROWS.audio}px` }"
        >
          <SpaceVideoTimelineAudio
            v-for="clip in project.audioTrack.clips.filter((entry) => entry.lane === lane - 1)"
            :key="clip.id"
            :clip="clip"
            :editor="editor"
          />
        </div>
        <span
          v-if="gestures.snapMs.value !== undefined"
          class="pointer-events-none absolute inset-y-0 z-20 w-px bg-surface-accent"
          :style="{ transform: `translateX(${px(gestures.snapMs.value)}px)` }"
        />
        <SpaceVideoPlayhead :time-ms="engine.timeMs.value" :px-per-second="timeline.pxPerSecond.value" />
        <button
          v-if="selection.selection.value"
          type="button"
          class="absolute inset-0 -z-10 w-full cursor-default"
          :aria-label="t('spaces.media.videoEditor.timeline.clearSelection')"
          @click="selection.clear()"
        />
      </div>
    </div>
  </div>
</template>
