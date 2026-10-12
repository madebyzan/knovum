<script setup lang="ts">
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { VideoMaterialNode } from "../material";
import type { VideoTextOverlay } from "../project";
import { clipTimelineStartMs } from "../timing";
import SpaceVideoAudioLane from "./SpaceVideoAudioLane.vue";
import SpaceVideoTimelineClip from "./SpaceVideoTimelineClip.vue";
import SpaceVideoTimelineLabels from "./SpaceVideoTimelineLabels.vue";
import SpaceVideoTimelineText from "./SpaceVideoTimelineText.vue";
import SpaceVideoTimeRuler from "./SpaceVideoTimeRuler.vue";
import SpaceVideoTransition from "./SpaceVideoTransition.vue";
import { dragVideoClip, videoTimelineDrop } from "./use-video-timeline-drop";
import { useVideoTimelineInteraction, type VideoTimelineProps } from "./use-video-timeline-interaction";
const props = defineProps<
  VideoTimelineProps & { spaceId: string; materials: ReadonlyMap<string, VideoMaterialNode> }
>();
const emit = defineEmits<{
  seek: [milliseconds: number];
  edit: [project: VideoTimelineProps["project"]];
  draft: [project: VideoTimelineProps["project"] | undefined];
  pause: [];
  selectClip: [id: string];
  selectText: [id: string];
  selectAudio: [id: string];
  addMaterial: [id: string, atMs: number];
}>();
const {
  visibleProject,
  viewport,
  durationMs,
  ticks,
  contentStyle,
  clipStyle,
  startScrub,
  moveScrub,
  startTrim,
  moveTrim,
  finishTrim,
  cancelTrim,
  nudgeTrim,
  escape,
} = useVideoTimelineInteraction(props, {
  seek: (value) => emit("seek", value),
  trim: (value) => emit("edit", value),
  selectClip: (id) => emit("selectClip", id),
  draft: (value) => emit("draft", value),
  pause: () => emit("pause"),
});
function editText(edited: VideoTextOverlay) {
  emit("edit", {
    ...props.project,
    textTrack: {
      items: props.project.textTrack.items.map((item) => (item.id === edited.id ? edited : item)),
    },
  });
}
function drop(event: DragEvent) {
  videoTimelineDrop(event, props.project, props.playheadMs, {
    edit: (value) => emit("edit", value),
    add: (id, atMs) => emit("addMaterial", id, atMs),
  });
}
function asset(id: string) {
  return visibleProject.value.assets.find(({ nodeId }) => nodeId === id)!;
}
</script>
<template>
  <UiScrollArea class="h-full" @keydown.esc.capture="escape">
    <section
      class="grid min-h-full grid-cols-[5rem_minmax(0,1fr)] bg-surface-panel"
      :aria-label="$t('spaces.media.player.timeline')"
      @dragover.prevent
      @drop="drop"
    >
      <SpaceVideoTimelineLabels :audio-count="project.audioTrack.clips.length" />
      <div
        :ref="(value) => (viewport = value as HTMLElement | null)"
        class="min-w-0 overflow-x-auto overflow-y-hidden"
      >
        <div class="relative min-h-full min-w-full" :style="contentStyle">
          <SpaceVideoTimeRuler
            :project="project"
            :duration-ms="durationMs"
            :playhead-ms="playheadMs"
            :ticks="ticks"
            @seek="emit('seek', $event)"
            @scrub-start="startScrub"
            @scrub-move="moveScrub"
          />
          <div data-text-track class="relative h-11 border-b border-stroke-divider bg-surface-canvas/40">
            <SpaceVideoTimelineText
              v-for="item in visibleProject.textTrack.items"
              :key="item.id"
              :item="item"
              :duration-ms="durationMs"
              :selected="selectedTextId === item.id"
              @select="emit('selectText', item.id)"
              @edit="editText"
            />
          </div>
          <div data-video-track class="relative h-24 border-b border-stroke-divider">
            <div
              v-for="(clip, index) in visibleProject.videoTrack.clips"
              :key="clip.id"
              class="absolute inset-y-0"
              :style="{
                ...clipStyle(clip.endMs - clip.startMs),
                left: `${(clipTimelineStartMs(visibleProject, clip.id) / durationMs) * 100}%`,
              }"
              :data-clip-id="clip.id"
              draggable="true"
              @dragstart="dragVideoClip($event, clip.id)"
            >
              <SpaceVideoTimelineClip
                :clip="clip"
                :selected="selectedClipId === clip.id"
                :source-path="materials.get(clip.assetId)?.data.assetPath"
                :asset="asset(clip.assetId)"
                :space-id="spaceId"
                :title="
                  materials.get(clip.assetId)?.data.title ??
                  $t('spaces.media.videoEditor.materials.unavailable')
                "
                :frame-rate="project.output.frameRate"
                :waveform="clip.assetId === project.assets[0]!.nodeId ? waveform : undefined"
                @select="emit('selectClip', clip.id)"
                @trim-start="(event, edge) => startTrim(event, clip, edge)"
                @trim-move="moveTrim"
                @trim-end="finishTrim"
                @trim-cancel="cancelTrim"
                @nudge="(edge, delta) => nudgeTrim(clip, edge, delta)"
              />
              <div
                v-if="index < visibleProject.videoTrack.clips.length - 1"
                class="absolute top-5 z-20 -translate-x-1/2"
                :style="{
                  left: `${100 - ((clip.transition?.durationMs ?? 0) / (2 * (clip.endMs - clip.startMs))) * 100}%`,
                }"
              >
                <SpaceVideoTransition
                  :project="project"
                  :clip="clip"
                  :index="index"
                  @edit="emit('edit', $event)"
                />
              </div>
            </div>
          </div>
          <SpaceVideoAudioLane
            v-for="clip in visibleProject.audioTrack.clips"
            :key="clip.id"
            :clip="clip"
            :title="
              materials.get(clip.assetId)?.data.title ?? $t('spaces.media.videoEditor.materials.unavailable')
            "
            :duration-ms="durationMs"
            :selected="selectedAudioId === clip.id"
            @select="emit('selectAudio', clip.id)"
          />
          <div
            v-if="!project.audioTrack.clips.length"
            class="flex h-8 items-center px-3 text-caption-sm text-fg-tertiary"
          >
            {{ $t("spaces.media.videoEditor.materials.audioHint") }}
          </div>
          <span
            class="pointer-events-none absolute inset-y-0 z-30 w-px bg-surface-danger"
            :style="{ left: `${(playheadMs / durationMs) * 100}%` }"
            ><span class="absolute top-0 -left-1 size-2 rotate-45 bg-surface-danger"
          /></span>
        </div>
      </div>
    </section>
  </UiScrollArea>
</template>
