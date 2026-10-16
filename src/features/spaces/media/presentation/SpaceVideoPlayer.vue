<script setup lang="ts">
import { Film, Pause, Play } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { formatDurationMs } from "../format-duration";
import { versionedMediaSource } from "../media-element-source";
import type { SpaceMediaPlaybackState } from "../playback";
import { useMediaPlayback } from "../use-media-playback";
import { useVideoPlayerOverlay } from "../use-video-player-overlay";
import SpaceMediaControls from "./SpaceMediaControls.vue";

const props = defineProps<{ data: SpaceVideoPayload }>();
const emit = defineEmits<{
  playbackError: [message: string];
  playback: [state: SpaceMediaPlaybackState];
  unload: [];
}>();
const { t } = useI18n();
const mediaElement = useTemplateRef<HTMLVideoElement>("mediaElement");
const source = computed(() => versionedMediaSource(props.data.assetPath, props.data.sourceRevision));
const cover = computed(() => versionedMediaSource(props.data.previewPath, props.data.sourceRevision));
const coverDuration = computed(() =>
  props.data.durationSeconds ? formatDurationMs(props.data.durationSeconds * 1000) : undefined,
);

const { loaded, playing, currentMs, durationMs, togglePlayback, seekTo, seekFromControl, unload } =
  useMediaPlayback(mediaElement, {
    sourceRevision: () => props.data.sourceRevision,
    onPlayback: (state) => emit("playback", state),
    onUnload: () => emit("unload"),
  });

const { controlsVisible, flashFeedback, handlePointerLeave, triggerFlash, wakeControls } =
  useVideoPlayerOverlay(playing);

const progressRatio = computed(() =>
  durationMs.value > 0 ? Math.min(currentMs.value / durationMs.value, 1) : 0,
);

function requestPlayback() {
  void togglePlayback().catch((error) => emit("playbackError", getErrorMessage(error)));
}

function handleStageClick() {
  triggerFlash(playing.value ? "pause" : "play");
  requestPlayback();
}

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div class="relative flex min-h-0 flex-1 overflow-hidden bg-surface-control/25">
    <div
      v-if="!loaded"
      class="relative block size-full overflow-hidden bg-surface-control/20 text-fg-secondary"
    >
      <img
        v-if="cover"
        :src="cover"
        :alt="data.title"
        draggable="false"
        class="block size-full object-contain"
      />
      <span v-else class="grid size-full place-items-center bg-surface-control text-fg-tertiary">
        <Film class="size-8" :stroke-width="1.35" />
      </span>
      <span class="absolute inset-0 bg-surface-scrim/5" />
      <span
        v-if="coverDuration"
        class="absolute right-2 bottom-2 rounded-md bg-surface-overlay/94 px-1.5 py-0.5 text-caption-sm font-medium text-fg-primary tabular-nums shadow-control ring-1 ring-stroke-default"
      >
        {{ coverDuration }}
      </span>
      <button
        type="button"
        class="nodrag pointer-events-auto absolute top-1/2 left-1/2 grid size-10 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full border border-white/20 bg-black/60 text-white shadow-dialog backdrop-blur-sm transition-transform duration-150 ease-standard hover:scale-105 active:scale-95"
        :aria-label="t('spaces.media.player.load', { title: data.title })"
        @click.stop="requestPlayback"
      >
        <Play class="ml-0.5 size-4" fill="currentColor" :stroke-width="1.4" />
      </button>
    </div>
    <div
      v-else
      class="nodrag pointer-events-auto relative size-full cursor-pointer overflow-hidden bg-surface-control select-none"
      @click="handleStageClick"
      @pointermove="wakeControls"
      @pointerleave="handlePointerLeave"
    >
      <video
        ref="mediaElement"
        :src="source"
        preload="metadata"
        class="pointer-events-none block size-full object-contain"
        @ended="unload"
      />
      <Transition
        enter-active-class="transition duration-150 ease-out"
        enter-from-class="opacity-0 scale-75"
        leave-active-class="transition duration-300 ease-in"
        leave-to-class="opacity-0 scale-125"
      >
        <div
          v-if="flashFeedback"
          class="pointer-events-none absolute top-1/2 left-1/2 grid size-12 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full bg-black/60 text-white shadow-dialog backdrop-blur-sm"
        >
          <Pause v-if="flashFeedback === 'pause'" class="size-6" fill="currentColor" />
          <Play v-else class="ml-0.5 size-6" fill="currentColor" />
        </div>
      </Transition>
      <div
        class="pointer-events-none absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-black/60 via-black/20 to-transparent transition-opacity duration-300 ease-standard"
        :class="controlsVisible ? 'opacity-100' : 'opacity-0'"
      />
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 translate-y-1.5"
        leave-active-class="transition duration-300 ease-in"
        leave-to-class="opacity-0 translate-y-1.5"
      >
        <div
          v-if="controlsVisible"
          class="nodrag pointer-events-auto absolute inset-x-3 bottom-2.5 z-10 flex items-center justify-center"
          @click.stop
        >
          <SpaceMediaControls
            class="w-full max-w-md"
            :playing="playing"
            :current-ms="currentMs"
            :duration-ms="durationMs"
            @toggle="requestPlayback"
            @seek="seekFromControl"
          />
        </div>
      </Transition>
      <div
        v-if="!controlsVisible"
        class="pointer-events-none absolute inset-x-0 bottom-0 h-0.5 w-full bg-white/20 transition-opacity duration-200"
      >
        <div
          class="h-full bg-white transition-[width] duration-100 ease-linear"
          :style="{ width: `${progressRatio * 100}%` }"
        />
      </div>
    </div>
  </div>
</template>
