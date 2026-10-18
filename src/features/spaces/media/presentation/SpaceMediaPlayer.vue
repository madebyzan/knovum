<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import type { SpaceMediaPlaybackState } from "../playback";
import SpaceAudioPlayer from "./SpaceAudioPlayer.vue";
import SpaceVideoPlayer from "./SpaceVideoPlayer.vue";

const props = withDefaults(defineProps<{ data: SpaceMediaPayload; interactive?: boolean }>(), {
  interactive: false,
});
const emit = defineEmits<{
  playbackError: [message: string];
  playback: [state: SpaceMediaPlaybackState];
  unload: [];
}>();
const audioPlayer = useTemplateRef<InstanceType<typeof SpaceAudioPlayer>>("audioPlayer");
const videoPlayer = useTemplateRef<InstanceType<typeof SpaceVideoPlayer>>("videoPlayer");
const isAudio = computed(() => props.data.kind === "audio");

function seekTo(positionMs: number) {
  return isAudio.value ? audioPlayer.value?.seekTo(positionMs) : videoPlayer.value?.seekTo(positionMs);
}

function togglePlayback() {
  return isAudio.value ? audioPlayer.value?.togglePlayback() : videoPlayer.value?.togglePlayback();
}

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-surface-control/25">
    <SpaceAudioPlayer
      v-if="data.kind === 'audio'"
      ref="audioPlayer"
      :data="data"
      :interactive="interactive"
      @playback="emit('playback', $event)"
      @unload="emit('unload')"
      @playback-error="emit('playbackError', $event)"
    />
    <SpaceVideoPlayer
      v-else
      ref="videoPlayer"
      :data="data"
      @playback="emit('playback', $event)"
      @unload="emit('unload')"
      @playback-error="emit('playbackError', $event)"
    />
  </div>
</template>
