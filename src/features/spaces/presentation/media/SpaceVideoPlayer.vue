<script setup lang="ts">
import { Film, Play } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { formatDurationMs } from "../../media/format-duration";
import type { SpaceMediaPlaybackState } from "../../media/playback";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { versionedMediaSource } from "./media-element-source";
import SpaceMediaControls from "./SpaceMediaControls.vue";
import { useMediaPlayback } from "./use-media-playback";

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
    onPlaybackError: (msg) => emit("playbackError", msg),
    onUnload: () => emit("unload"),
  });

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div class="pointer-events-none relative flex min-h-0 flex-1 overflow-hidden bg-control/25">
    <div
      v-if="!loaded"
      class="relative block size-full overflow-hidden bg-control/20 text-foreground-secondary"
    >
      <img
        v-if="cover"
        :src="cover"
        :alt="data.title"
        draggable="false"
        class="block size-full object-contain"
      />
      <span v-else class="grid size-full place-items-center bg-control text-foreground-tertiary">
        <Film class="size-8" :stroke-width="1.35" />
      </span>
      <span class="absolute inset-0 bg-scrim/5" />
      <span
        v-if="coverDuration"
        class="absolute right-2 bottom-2 rounded-control bg-overlay/94 px-1.5 py-0.5 text-micro font-medium text-foreground tabular-nums shadow-control ring-1 ring-border"
        >{{ coverDuration }}</span
      >
      <button
        type="button"
        class="nodrag pointer-events-auto absolute top-1/2 left-1/2 grid size-9 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full bg-overlay/94 text-foreground shadow-control ring-1 ring-border"
        :aria-label="t('spaces.media.player.load', { title: data.title })"
        @click.stop="togglePlayback"
      >
        <Play class="ml-px size-(--knovum-icon-size-md)" fill="currentColor" :stroke-width="1.4" />
      </button>
    </div>
    <div v-else class="relative size-full overflow-hidden bg-control">
      <video
        ref="mediaElement"
        :src="source"
        preload="metadata"
        class="block size-full object-contain"
        @ended="unload"
      />
    </div>
    <SpaceMediaControls
      v-if="loaded"
      floating
      class="absolute inset-x-1.5 bottom-1.5 z-10"
      :playing="playing"
      :current-ms="currentMs"
      :duration-ms="durationMs"
      @toggle="togglePlayback"
      @seek="seekFromControl"
      @close="unload"
    />
  </div>
</template>
