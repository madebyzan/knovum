<script setup lang="ts">
import { Pause, Play } from "@lucide/vue";
import { inject, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatDurationMs } from "../../media/format-duration";
import SpaceMediaTimeline from "../../media/presentation/SpaceMediaTimeline.vue";
import { useMediaPlayback } from "../../media/use-media-playback";

const props = withDefaults(
  defineProps<{
    source: string;
    kind: "audio" | "video";
    preview?: boolean;
    showControls?: boolean;
    label?: string;
  }>(),
  { preview: true, showControls: true },
);
const emit = defineEmits<{
  ended: [];
  play: [];
  pause: [];
  loaded: [durationMs: number];
  error: [message: string | undefined];
}>();
const { t } = useI18n();
const element = useTemplateRef<HTMLMediaElement>("element");
const error = shallowRef<string>();
const stopAt = shallowRef<number>();
watch(error, (message) => emit("error", message));
const playback = useMediaPlayback(element, {
  sourceRevision: () => props.source,
  onUnload: () => {
    stopAt.value = undefined;
  },
});
watch(playback.currentMs, (position) => {
  if (stopAt.value !== undefined && position >= stopAt.value) {
    pause();
  }
});
const scopeVisible = inject(overlayScopeVisibleKey, undefined);
watch(
  () => scopeVisible?.value,
  (visible) => {
    if (visible === false) pause();
  },
);
function pause() {
  element.value?.pause();
  playback.playing.value = false;
  stopAt.value = undefined;
}
async function seekTo(startMs: number, endMs?: number) {
  try {
    error.value = undefined;
    stopAt.value = endMs;
    await playback.seekTo(startMs);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
async function toggle() {
  try {
    error.value = undefined;
    stopAt.value = undefined;
    await playback.togglePlayback();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
function loaded(event: Event) {
  const duration = (event.target as HTMLMediaElement).duration;
  if (Number.isFinite(duration)) emit("loaded", Math.round(duration * 1000));
}
function seek(position: number) {
  stopAt.value = undefined;
  playback.seekFromControl(position);
}
onBeforeUnmount(() => element.value?.pause());
defineExpose({ seekTo, pause });
</script>

<template>
  <div class="min-w-0 space-y-2" role="group" :aria-label="label">
    <component
      :is="kind === 'video' ? 'video' : 'audio'"
      ref="element"
      :class="
        kind === 'video' && preview
          ? 'max-h-40 w-full rounded-md bg-surface-control object-contain'
          : 'hidden'
      "
      :src="source"
      preload="metadata"
      @loadedmetadata="loaded"
      @error="error = t('spaces.media.player.playbackFailed')"
      @play="emit('play')"
      @pause="emit('pause')"
      @ended="emit('ended')"
    />
    <div
      v-if="showControls"
      class="flex items-center gap-2 rounded-md border border-stroke-divider bg-surface-control px-2 py-1"
    >
      <UiIconButton
        :label="playback.playing.value ? t('common.pause') : t('spaces.media.player.play')"
        size="sm"
        @click="toggle"
      >
        <Pause v-if="playback.playing.value" class="size-3.5" fill="currentColor" />
        <Play v-else class="size-3.5" fill="currentColor" />
      </UiIconButton>
      <SpaceMediaTimeline
        :current-ms="playback.currentMs.value"
        :duration-ms="playback.durationMs.value"
        @seek="seek"
      />
      <span class="shrink-0 text-caption-sm text-fg-secondary tabular-nums">
        {{ formatDurationMs(playback.currentMs.value) }} / {{ formatDurationMs(playback.durationMs.value) }}
      </span>
    </div>
    <p v-if="error && showControls" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
  </div>
</template>
