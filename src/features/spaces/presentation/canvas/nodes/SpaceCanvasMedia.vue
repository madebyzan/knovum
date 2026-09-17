<script setup lang="ts">
import { storeToRefs } from "pinia";
import { nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { SpaceMediaPayload } from "../../../model/nodes/capabilities";
import { useSpaceStore } from "../../../session/space-store";
import SpaceMediaPlayer from "../../media/SpaceMediaPlayer.vue";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

const props = defineProps<{ id: string; data: SpaceMediaPayload }>();
const store = useSpaceStore();
const { t } = useI18n();
const { mediaSeekRequest, mediaToggleRequest } = storeToRefs(store);
const player = useTemplateRef<InstanceType<typeof SpaceMediaPlayer>>("player");

watch([mediaToggleRequest, player], ([request]) => void consumeToggleRequest(request), {
  immediate: true,
  flush: "post",
});
watch([mediaSeekRequest, player], ([request]) => void consumeSeekRequest(request), {
  immediate: true,
  flush: "post",
});

async function consumeToggleRequest(request: typeof mediaToggleRequest.value) {
  if (!request || request.id !== props.id || !player.value) return;
  await nextTick();
  try {
    await player.value.togglePlayback();
  } catch (error) {
    toast.error(String(error));
  } finally {
    store.consumeMediaToggle(request.nonce);
  }
}

async function consumeSeekRequest(request: typeof mediaSeekRequest.value) {
  if (!request || request.id !== props.id || !player.value) return;
  await nextTick();
  try {
    await player.value.seekTo(request.positionMs);
  } catch (error) {
    toast.error(String(error));
  } finally {
    store.consumeMediaSeek(request.nonce);
  }
}
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
    <SpaceCanvasProcessingState
      v-if="data.processingState === 'importing'"
      :label="t('spaces.transfer.importingItem')"
    />
    <SpaceMediaPlayer
      v-else
      ref="player"
      :data="data"
      @playback="store.updateMediaPlayback(id, $event)"
      @unload="store.clearMediaPlayback(id)"
      @playback-error="toast.error($event)"
    />
  </div>
</template>
