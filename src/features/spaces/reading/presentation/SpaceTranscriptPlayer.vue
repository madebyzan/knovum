<script setup lang="ts">
import { Clock } from "@lucide/vue";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { formatDurationMs } from "../../media/format-duration";

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
      class="block max-h-56 w-full rounded-lg bg-surface-stage object-contain"
      @timeupdate="onTimeUpdate"
    />
    <audio
      v-else
      ref="mediaElement"
      :src="mediaSrc"
      controls
      preload="metadata"
      class="block w-full"
      @timeupdate="onTimeUpdate"
    />
    <div v-if="chapters.length" class="mt-2.5 flex flex-wrap gap-1">
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
    <UiSearchInput
      v-if="hasSegments"
      v-model="query"
      class="mt-2.5 w-full"
      :placeholder="t('spaces.transcript.text.search')"
    />
  </div>
</template>
