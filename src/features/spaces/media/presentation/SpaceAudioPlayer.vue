<script setup lang="ts">
import { Pause, Play, X } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import { formatDurationMs } from "../format-duration";
import { versionedMediaSource } from "../media-element-source";
import type { SpaceMediaPlaybackState } from "../playback";
import { mediaProviderDisplayName } from "../provider-display-name";
import { useMediaPlayback } from "../use-media-playback";

const WAVEFORM_BAR_COUNT = 52;

const props = withDefaults(defineProps<{ data: SpaceMediaPayload; interactive?: boolean }>(), {
  interactive: false,
});
const emit = defineEmits<{
  playbackError: [message: string];
  playback: [state: SpaceMediaPlaybackState];
  unload: [];
}>();
const { t } = useI18n();

const mediaElement = useTemplateRef<HTMLMediaElement>("mediaElement");
const source = computed(() => versionedMediaSource(props.data.assetPath, props.data.sourceRevision));
const analysis = computed(() => {
  const current = props.data.mediaAnalysis;
  return current?.sourceRevision === props.data.sourceRevision ? current : undefined;
});
const waveform = computed(() => compactWaveform(analysis.value?.waveform));

const { loaded, playing, currentMs, togglePlayback, seekTo, seekFromControl, unload } = useMediaPlayback(
  mediaElement,
  {
    sourceRevision: () => props.data.sourceRevision,
    onPlayback: (state) => emit("playback", state),
    onUnload: () => emit("unload"),
  },
);

const durationLabel = computed(() => (analysis.value ? formatDurationMs(analysis.value.durationMs) : "—"));
const playedRatio = computed(() =>
  analysis.value ? Math.min(currentMs.value / analysis.value.durationMs, 1) : 0,
);
const seekable = computed(() => Boolean(props.interactive && loaded.value));

function requestPlayback() {
  void togglePlayback().catch((error) => emit("playbackError", getErrorMessage(error)));
}

function seekFromSlider(event: Event) {
  seekFromControl(Number((event.target as HTMLInputElement).value));
}

function compactWaveform(samples?: number[]) {
  if (!samples?.length) return [];
  const count = Math.min(samples.length, WAVEFORM_BAR_COUNT);
  const bucketSize = samples.length / count;
  const peaks = Array.from({ length: count }, (_, index) => {
    const start = Math.floor(index * bucketSize);
    const end = Math.max(start + 1, Math.floor((index + 1) * bucketSize));
    return Math.max(...samples.slice(start, end));
  });
  const maximum = Math.max(...peaks);
  return maximum > 0 ? peaks.map((peak) => Math.sqrt(peak / maximum)) : peaks;
}

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div
    class="pointer-events-none grid min-h-0 flex-1 grid-cols-[2.25rem_minmax(0,1fr)] grid-rows-[1.25rem_2.25rem] content-center gap-x-3 gap-y-1 px-3 py-2 select-none"
  >
    <UiIconButton
      class="nodrag pointer-events-auto row-span-2 size-9 self-center rounded-full border border-stroke-strong bg-surface-raised text-fg-primary shadow-control hover:bg-surface-control-hover"
      :label="playing ? t('common.pause') : t('spaces.media.player.play')"
      size="sm"
      @click.stop="requestPlayback"
    >
      <Pause v-if="playing" class="size-3.5" fill="currentColor" />
      <Play v-else class="ml-px size-3.5" fill="currentColor" />
    </UiIconButton>

    <div class="flex min-w-0 items-center gap-2">
      <strong class="min-w-0 flex-1 truncate text-label font-medium">{{ data.title }}</strong>
      <span v-if="data.origin" class="max-w-16 shrink-0 truncate text-caption-sm text-fg-tertiary">
        {{ mediaProviderDisplayName(data.origin.provider) }}
      </span>
      <span class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums">
        <template v-if="loaded">{{ formatDurationMs(currentMs) }} / </template>{{ durationLabel }}
      </span>
      <UiIconButton
        v-if="loaded"
        class="nodrag pointer-events-auto -mr-1 size-6 shrink-0"
        :label="t('spaces.media.player.closePlayback')"
        size="sm"
        @click.stop="unload"
      >
        <X class="size-3" />
      </UiIconButton>
    </div>

    <div class="relative min-w-0 rounded-sm focus-within:ring-1 focus-within:ring-stroke-accent">
      <span v-if="analysis" class="audio-waveform" aria-hidden="true">
        <span
          v-for="(amplitude, index) in waveform"
          :key="index"
          class="audio-waveform-bar"
          :class="(index + 1) / waveform.length <= playedRatio && 'audio-waveform-bar-played'"
          :style="{ height: `${Math.round(5 + amplitude * 27)}px` }"
        />
      </span>
      <span v-else class="flex h-full items-center text-caption-sm text-fg-tertiary" role="status">
        {{ t("spaces.media.analysis.unavailable") }}
      </span>
      <input
        v-if="analysis"
        class="nodrag absolute inset-0 size-full opacity-0"
        :class="seekable ? 'pointer-events-auto cursor-pointer' : 'pointer-events-none cursor-default'"
        type="range"
        min="0"
        :max="analysis.durationMs"
        :value="currentMs"
        :disabled="!seekable"
        :aria-label="t('spaces.media.player.timeline')"
        @input="seekFromSlider"
      />
    </div>

    <audio v-if="loaded" ref="mediaElement" :src="source" preload="metadata" class="hidden" @ended="unload" />
  </div>
</template>

<style scoped>
.audio-waveform {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.audio-waveform-bar {
  width: 2px;
  flex: 0 0 2px;
  border-radius: 999px;
  background: color-mix(in srgb, currentColor 28%, transparent);
}

.audio-waveform-bar-played {
  background: var(--color-fg-accent);
}
</style>
