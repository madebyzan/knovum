<script setup lang="ts">
import { shallowRef, useTemplateRef, watch } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useMediaPlayback } from "../../media/use-media-playback";

const props = defineProps<{ source: string; kind: "audio" | "video" }>();
const emit = defineEmits<{ ended: []; loaded: [durationMs: number] }>();
const element = useTemplateRef<HTMLMediaElement>("element");
const error = shallowRef<string>();
const stopAt = shallowRef<number>();
const playback = useMediaPlayback(element, {
  sourceRevision: () => props.source,
  onUnload: () => {
    stopAt.value = undefined;
  },
});
watch(playback.currentMs, (position) => {
  if (stopAt.value !== undefined && position >= stopAt.value) {
    playback.playing.value = false;
    stopAt.value = undefined;
  }
});
async function seekTo(startMs: number, endMs?: number) {
  try {
    error.value = undefined;
    stopAt.value = endMs;
    await playback.seekTo(startMs);
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
function loaded(event: Event) {
  const duration = (event.target as HTMLMediaElement).duration;
  if (Number.isFinite(duration)) emit("loaded", Math.round(duration * 1000));
}
defineExpose({ seekTo });
</script>

<template>
  <div class="min-w-0 flex-1">
    <video
      v-if="kind === 'video'"
      ref="element"
      class="w-full rounded"
      controls
      :src="source"
      @loadedmetadata="loaded"
      @ended="emit('ended')"
    />
    <audio
      v-else
      ref="element"
      class="h-8 w-full"
      controls
      :src="source"
      @loadedmetadata="loaded"
      @ended="emit('ended')"
    />
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
  </div>
</template>
