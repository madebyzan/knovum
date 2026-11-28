<script setup lang="ts">
import { Pause, Play, RotateCcw, RotateCw, Volume2, VolumeX } from "@lucide/vue";
import { computed, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { formatDurationMs } from "../../media/format-duration";
import { versionedMediaSource } from "../../media/media-element-source";
import type { SpaceMediaPlaybackState } from "../../media/playback";
import SpaceMediaTimeline from "../../media/presentation/SpaceMediaTimeline.vue";
import { useMediaPlayback } from "../../media/use-media-playback";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../../model/nodes/node-payload";

const props = withDefaults(
  defineProps<{
    media: SpaceVideoPayload | SpaceAudioPayload;
    hasSegments: boolean;
    visible?: boolean;
  }>(),
  { visible: true },
);

const query = defineModel<string>("query", { required: true });
const emit = defineEmits<{ timeUpdate: [ms: number]; playback: [state: SpaceMediaPlaybackState] }>();
const { t } = useI18n();
const mediaElement = useTemplateRef<HTMLMediaElement>("mediaElement");
const searchWrapper = useTemplateRef<HTMLElement>("searchWrapper");

const mediaSrc = computed(() => versionedMediaSource(props.media.assetPath, props.media.sourceRevision));
const posterSrc = computed(() =>
  props.media.kind === "video"
    ? versionedMediaSource(props.media.previewPath, props.media.sourceRevision)
    : undefined,
);
const fallbackDurationMs = computed(() => {
  if (props.media.mediaAnalysis?.durationMs) return props.media.mediaAnalysis.durationMs;
  if (props.media.kind === "video" && props.media.durationSeconds) return props.media.durationSeconds * 1_000;
  return 0;
});

const {
  loaded,
  playing,
  currentMs,
  durationMs,
  rate,
  muted,
  togglePlayback: rawToggle,
  pause,
  seekTo: rawSeekTo,
  seekFromControl,
  skipBy,
  cycleRate,
  toggleMute,
} = useMediaPlayback(mediaElement, {
  sourceRevision: () => props.media.sourceRevision,
  onPlayback: (state) => {
    emit("timeUpdate", state.currentMs);
    emit("playback", state);
  },
  onUnload: () => {
    loaded.value = true;
  },
});

loaded.value = true;
const effectiveDurationMs = computed(() => durationMs.value || fallbackDurationMs.value);

watch(
  () => props.visible,
  (visible) => {
    if (!visible && playing.value) pause();
  },
);

async function seekTo(startMs: number, autoplay = true) {
  try {
    await rawSeekTo(startMs, autoplay);
  } catch {
    toast.error(t("spaces.media.player.playbackFailed"));
  }
}

async function togglePlayback() {
  try {
    await rawToggle();
  } catch {
    toast.error(t("spaces.media.player.playbackFailed"));
  }
}

function focusSearch() {
  searchWrapper.value?.querySelector<HTMLInputElement>("input")?.focus();
}

defineExpose({ seekTo, togglePlayback, skipBy, pause, focusSearch });
</script>

<template>
  <div
    data-reader-ignore-selection
    class="shrink-0 space-y-2 border-b border-stroke-divider/70 bg-surface-panel p-3 select-none"
  >
    <video
      v-if="media.kind === 'video'"
      ref="mediaElement"
      :src="mediaSrc"
      :poster="posterSrc"
      preload="metadata"
      class="block aspect-video max-h-56 w-full cursor-pointer rounded-lg bg-surface-stage object-contain"
      @click="togglePlayback"
    />
    <audio v-else ref="mediaElement" :src="mediaSrc" preload="metadata" class="hidden" />

    <div
      class="flex items-center gap-1.5 rounded-lg border border-stroke-divider bg-surface-raised px-2 py-1.5"
    >
      <UiIconButton
        size="sm"
        tooltip
        :label="playing ? t('common.pause') : t('spaces.media.player.play')"
        @click="togglePlayback"
      >
        <Pause v-if="playing" class="size-3.5" fill="currentColor" />
        <Play v-else class="ml-px size-3.5" fill="currentColor" />
      </UiIconButton>
      <UiIconButton size="sm" tooltip :label="t('spaces.media.player.skipBack')" @click="skipBy(-10_000)">
        <RotateCcw class="size-3" />
      </UiIconButton>
      <UiIconButton size="sm" tooltip :label="t('spaces.media.player.skipForward')" @click="skipBy(10_000)">
        <RotateCw class="size-3" />
      </UiIconButton>
      <SpaceMediaTimeline
        tone="control"
        class="mx-1"
        :current-ms="currentMs"
        :duration-ms="effectiveDurationMs"
        @seek="seekFromControl"
      />
      <span class="shrink-0 font-mono text-caption-sm text-fg-tertiary tabular-nums">
        {{ formatDurationMs(currentMs) }} / {{ formatDurationMs(effectiveDurationMs) }}
      </span>
      <button
        type="button"
        class="shrink-0 rounded px-1.5 py-0.5 font-mono text-caption-sm font-medium text-fg-secondary tabular-nums transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        :aria-label="t('spaces.media.player.speed')"
        :title="t('spaces.media.player.speed')"
        @click="cycleRate"
      >
        {{ rate }}×
      </button>
      <UiIconButton
        size="sm"
        tooltip
        :label="muted ? t('spaces.media.player.unmute') : t('spaces.media.player.mute')"
        @click="toggleMute"
      >
        <VolumeX v-if="muted" class="size-3.5" />
        <Volume2 v-else class="size-3.5" />
      </UiIconButton>
    </div>

    <div v-if="hasSegments" ref="searchWrapper" class="flex items-center gap-1.5">
      <UiSearchInput
        v-model="query"
        size="sm"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.transcript.text.search')"
      />
      <slot name="actions" />
    </div>
  </div>
</template>
