<script setup lang="ts">
import { Pause, Play, X } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceMediaPlaybackState } from "../../media/playback";
import { mediaProviderDisplayName } from "../../media/provider-display-name";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import { versionedMediaSource } from "./element";
import { useMediaPlayback } from "./use-media-playback";

const WAVEFORM_BAR_COUNT = 52;

const props = defineProps<{ data: SpaceMediaPayload }>();
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
    onPlaybackError: (msg) => emit("playbackError", msg),
    onUnload: () => emit("unload"),
  },
);

const durationLabel = computed(() => (analysis.value ? formatDurationMs(analysis.value.durationMs) : "—"));
const playedRatio = computed(() =>
  analysis.value ? Math.min(currentMs.value / analysis.value.durationMs, 1) : 0,
);

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
      class="nodrag pointer-events-auto row-span-2 size-9 self-center rounded-full border border-border-strong bg-raised text-foreground shadow-control hover:bg-control-hover"
      :label="playing ? t('common.pause') : t('spaces.media.player.play')"
      size="sm"
      @click.stop="togglePlayback"
    >
      <Pause v-if="playing" class="size-(--knovum-icon-size-sm)" fill="currentColor" />
      <Play v-else class="ml-px size-(--knovum-icon-size-sm)" fill="currentColor" />
    </UiIconButton>

    <div class="flex min-w-0 items-center gap-2">
      <strong class="min-w-0 flex-1 truncate text-label font-medium">{{ data.title }}</strong>
      <span v-if="data.origin" class="max-w-16 shrink-0 truncate text-micro text-foreground-tertiary">
        {{ mediaProviderDisplayName(data.origin.provider) }}
      </span>
      <span class="shrink-0 text-micro text-foreground-tertiary tabular-nums">
        <template v-if="loaded">{{ formatDurationMs(currentMs) }} / </template>{{ durationLabel }}
      </span>
      <UiIconButton
        v-if="loaded"
        class="nodrag pointer-events-auto -mr-1 size-6 shrink-0"
        :label="t('spaces.media.player.closePlayback')"
        size="sm"
        @click.stop="unload"
      >
        <X class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
    </div>

    <div class="relative min-w-0 rounded-sm focus-within:ring-1 focus-within:ring-accent">
      <span v-if="analysis" class="audio-waveform" aria-hidden="true">
        <span
          v-for="(amplitude, index) in waveform"
          :key="index"
          class="audio-waveform-bar"
          :class="(index + 1) / waveform.length <= playedRatio && 'audio-waveform-bar-played'"
          :style="{ height: `${Math.round(5 + amplitude * 27)}px` }"
        />
      </span>
      <span v-else class="flex h-full items-center text-micro text-foreground-tertiary" role="status">
        {{ t("spaces.media.analysis.unavailable") }}
      </span>
      <input
        v-if="analysis"
        class="nodrag pointer-events-auto absolute inset-0 size-full cursor-pointer opacity-0 disabled:cursor-default"
        type="range"
        min="0"
        :max="analysis.durationMs"
        :value="currentMs"
        :disabled="!loaded"
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
  background: var(--color-accent);
}
</style>
