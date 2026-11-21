<script setup lang="ts">
import { Clock } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { formatDurationMs } from "../../media/format-duration";

const PLAYBACK_RATES = [1, 1.25, 1.5, 2] as const;

defineProps<{
  kind: "video" | "audio";
  mediaSrc: string;
  chapters: readonly { startMs: number; title: string }[];
  hasSegments: boolean;
}>();

const query = defineModel<string>("query", { required: true });
const emit = defineEmits<{ seek: [startMs: number]; timeUpdate: [ms: number] }>();
const { t } = useI18n();
const mediaElement = useTemplateRef<HTMLMediaElement>("mediaElement");
const rateIndex = shallowRef(0);
const playbackRate = computed(() => PLAYBACK_RATES[rateIndex.value] ?? 1);

function applyPlaybackRate() {
  if (mediaElement.value) mediaElement.value.playbackRate = playbackRate.value;
}
function cyclePlaybackRate() {
  rateIndex.value = (rateIndex.value + 1) % PLAYBACK_RATES.length;
  applyPlaybackRate();
}
function seekTo(startMs: number) {
  if (mediaElement.value) mediaElement.value.currentTime = startMs / 1_000;
}
function togglePlayback() {
  const player = mediaElement.value;
  if (!player) return;
  if (player.paused) void player.play();
  else player.pause();
}
function onTimeUpdate(event: Event) {
  const target = event.target as HTMLMediaElement | null;
  if (target) emit("timeUpdate", Math.round(target.currentTime * 1_000));
}

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div
    data-reader-ignore-selection
    class="shrink-0 border-b border-stroke-divider bg-surface-panel p-3 select-none"
  >
    <video
      v-if="kind === 'video'"
      ref="mediaElement"
      :src="mediaSrc"
      controls
      preload="metadata"
      class="max-h-56 w-full rounded-lg bg-surface-inverse object-contain"
      @loadedmetadata="applyPlaybackRate"
      @timeupdate="onTimeUpdate"
    />
    <audio
      v-else
      ref="mediaElement"
      :src="mediaSrc"
      controls
      preload="metadata"
      class="w-full"
      @loadedmetadata="applyPlaybackRate"
      @timeupdate="onTimeUpdate"
    />
    <div v-if="chapters.length" class="mt-2 flex flex-wrap gap-1">
      <button
        v-for="chapter in chapters"
        :key="chapter.startMs"
        type="button"
        class="inline-flex items-center gap-1 rounded-md border border-stroke-divider bg-surface-raised px-2 py-0.5 text-caption-sm text-fg-secondary hover:bg-surface-control-hover"
        @click="emit('seek', chapter.startMs)"
      >
        <Clock class="size-2.5 text-fg-tertiary" />
        <span>{{ formatDurationMs(chapter.startMs) }}</span>
        <span class="max-w-32 truncate text-fg-primary">{{ chapter.title }}</span>
      </button>
    </div>
    <div class="mt-2 flex items-center gap-1.5">
      <UiSearchInput
        v-if="hasSegments"
        v-model="query"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.transcript.text.search')"
      />
      <button
        type="button"
        class="ml-auto shrink-0 rounded-md border border-stroke-divider bg-surface-raised px-2 py-1 font-mono text-caption-sm font-medium text-fg-secondary tabular-nums hover:bg-surface-control-hover hover:text-fg-primary"
        @click="cyclePlaybackRate"
      >
        {{ playbackRate }}×
      </button>
    </div>
  </div>
</template>
