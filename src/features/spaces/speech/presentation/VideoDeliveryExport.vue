<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import { useVideoDelivery } from "../use-video-delivery";
import { listVideoAudioTracks } from "../video-delivery-ipc";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";

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
function toggleTrack(revisionId: string, checked: boolean) {
  if (checked && !revisions.value.includes(revisionId)) revisions.value = [...revisions.value, revisionId];
  else if (!checked) revisions.value = revisions.value.filter((item) => item !== revisionId);
}
</script>

<template>
  <section class="space-y-2.5">
    <div class="flex flex-wrap gap-1.5">
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="delivery.busy.value"
        @click="delivery.exportAsset('srt')"
        >SRT</UiButton
      >
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="delivery.busy.value"
        @click="delivery.exportAsset('vtt')"
        >WebVTT</UiButton
      >
      <UiButton
        v-if="generated"
        size="sm"
        variant="secondary"
        :disabled="delivery.busy.value"
        @click="delivery.exportAsset('dialogue_wav')"
        >{{ t("spaces.speech.videoDubbing.dialogueStem") }}</UiButton
      >
      <UiButton
        v-if="generated && hasBackground"
        size="sm"
        variant="secondary"
        :disabled="delivery.busy.value"
        @click="delivery.exportAsset('background_wav')"
        >{{ t("spaces.speech.videoDubbing.backgroundStem") }}</UiButton
      >
    </div>
    <SpeechSettingsDisclosure
      v-if="tracks?.length"
      :title="t('spaces.speech.videoDubbing.multipleAudioTracks')"
      :summary="revisions.length ? String(revisions.length) : undefined"
    >
      <div class="space-y-2">
        <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.videoDubbing.audioTracksHint") }}</p>
        <div
          class="divide-y divide-stroke-divider overflow-hidden rounded-panel border border-stroke-default bg-surface-raised"
        >
          <label
            v-for="track in tracks"
            :key="track.revisionId"
            class="flex cursor-pointer items-center gap-2.5 px-2.5 py-2 text-caption text-fg-secondary"
          >
            <UiCheckbox
              :model-value="revisions.includes(track.revisionId)"
              :label="`${track.language} v${track.revision}`"
              :disabled="
                delivery.busy.value || (revisions.length >= 8 && !revisions.includes(track.revisionId))
              "
              @update:model-value="toggleTrack(track.revisionId, $event)"
            />
            <span>{{ track.language }} · v{{ track.revision }}</span>
          </label>
          <label class="flex cursor-pointer items-center gap-2.5 px-2.5 py-2 text-caption text-fg-secondary">
            <UiCheckbox
              v-model="retainOriginal"
              :label="t('spaces.speech.videoDubbing.retainOriginalTrack')"
              :disabled="delivery.busy.value"
            />
            <span>{{ t("spaces.speech.videoDubbing.retainOriginalTrack") }}</span>
          </label>
        </div>
        <UiButton
          size="sm"
          variant="secondary"
          :disabled="delivery.busy.value || !revisions.length"
          @click="delivery.exportTracks([...revisions], retainOriginal)"
          >{{ t("common.export") }} MP4</UiButton
        >
      </div>
    </SpeechSettingsDisclosure>
    <UiButton v-if="delivery.busy.value" size="sm" variant="dangerSubtle" @click="delivery.cancel">{{
      t("common.cancel")
    }}</UiButton>
    <p v-if="delivery.error.value || error" class="text-caption text-fg-danger">
      {{ delivery.error.value || error?.message }}
    </p>
  </section>
</template>
