<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useVideoDelivery } from "../../speech/use-video-delivery";
import { listVideoAudioTracks } from "../../speech/video-delivery-ipc";

const props = defineProps<{
  projectId: string;
  revisionId: string;
  title: string;
  generated: boolean;
  hasBackground: boolean;
}>();
const { t } = useI18n();
const delivery = useVideoDelivery(() => props);
const revisions = ref<string[]>([]);
const retainOriginal = shallowRef(true);
const { data: tracks, error } = useQuery({
  queryKey: computed(() => ["video-delivery-tracks", props.projectId, props.revisionId, props.generated]),
  queryFn: () => listVideoAudioTracks(props.projectId),
});
watch(
  () => props.revisionId,
  () => {
    revisions.value = [];
  },
);
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2">
    <h3 class="text-caption font-medium text-fg-secondary">{{ t("spaces.speech.videoDelivery") }}</h3>
    <div class="flex flex-wrap gap-2">
      <UiButton size="sm" :disabled="delivery.busy.value" @click="delivery.exportAsset('srt')">SRT</UiButton>
      <UiButton size="sm" :disabled="delivery.busy.value" @click="delivery.exportAsset('vtt')"
        >WebVTT</UiButton
      >
      <UiButton
        size="sm"
        :disabled="delivery.busy.value || !generated"
        @click="delivery.exportAsset('dialogue_wav')"
        >{{ t("spaces.speech.dialogueStem") }}</UiButton
      >
      <UiButton
        size="sm"
        :disabled="delivery.busy.value || !generated || !hasBackground"
        @click="delivery.exportAsset('background_wav')"
        >{{ t("spaces.speech.backgroundStem") }}</UiButton
      >
    </div>
    <details>
      <summary class="cursor-pointer text-caption text-fg-secondary">
        {{ t("spaces.speech.multipleAudioTracks") }}
      </summary>
      <div class="mt-2 space-y-2">
        <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.audioTracksHint") }}</p>
        <label
          v-for="track in tracks"
          :key="track.revisionId"
          class="flex gap-2 text-caption text-fg-secondary"
          ><input
            v-model="revisions"
            type="checkbox"
            :value="track.revisionId"
            :disabled="
              delivery.busy.value || (revisions.length >= 8 && !revisions.includes(track.revisionId))
            "
          />{{ track.language }} · v{{ track.revision }}</label
        >
        <label class="flex gap-2 text-caption text-fg-secondary"
          ><input v-model="retainOriginal" type="checkbox" :disabled="delivery.busy.value" />{{
            t("spaces.speech.retainOriginalTrack")
          }}</label
        >
        <UiButton
          size="sm"
          :disabled="delivery.busy.value || !revisions.length"
          @click="delivery.exportTracks([...revisions], retainOriginal)"
          >{{ t("spaces.speech.export") }} MP4</UiButton
        >
      </div>
    </details>
    <UiButton v-if="delivery.busy.value" size="sm" @click="delivery.cancel">{{
      t("spaces.speech.cancel")
    }}</UiButton>
    <p v-if="delivery.error.value || error" class="text-caption text-fg-danger">
      {{ delivery.error.value || error?.message }}
    </p>
  </section>
</template>
