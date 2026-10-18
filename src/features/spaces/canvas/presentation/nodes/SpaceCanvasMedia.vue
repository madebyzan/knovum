<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import SpaceMediaPlayer from "../../../media/presentation/SpaceMediaPlayer.vue";
import type { SpaceMediaPayload } from "../../../model/nodes/capabilities";
import { useSpaceStore } from "../../../session/space-store";
import { nodePayloadWork } from "./canvas-node-work";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

const props = withDefaults(defineProps<{ id: string; data: SpaceMediaPayload; interactive?: boolean }>(), {
  interactive: false,
});
const store = useSpaceStore();
const { t } = useI18n();
const { mediaSeekRequest, mediaToggleRequest } = storeToRefs(store);
const player = useTemplateRef<InstanceType<typeof SpaceMediaPlayer>>("player");
const work = computed(() => nodePayloadWork(props.data));

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
    toast.error(getErrorMessage(error));
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
    toast.error(getErrorMessage(error));
  } finally {
    store.consumeMediaSeek(request.nonce);
  }
}
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
    <SpaceMediaPlayer
      v-if="work?.surface !== 'content'"
      ref="player"
      :data="data"
      :interactive="interactive"
      @playback="store.updateMediaPlayback(id, $event)"
      @unload="store.clearMediaPlayback(id)"
      @playback-error="toast.error($event)"
    />
    <Transition
      leave-active-class="transition-opacity duration-220 motion-reduce:transition-none"
      leave-to-class="opacity-0"
    >
      <SpaceCanvasProcessingState
        v-if="work?.surface === 'content'"
        class="absolute inset-0"
        :label="t(work.labelKey)"
        :phase="work.phase"
      />
    </Transition>
  </div>
</template>
