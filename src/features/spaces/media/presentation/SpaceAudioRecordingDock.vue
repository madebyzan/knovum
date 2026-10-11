<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useSpaceStore } from "../../session/space-store";
import SpaceAudioRecorder from "./SpaceAudioRecorder.vue";

const store = useSpaceStore();
const { t } = useI18n();
const open = computed({
  get: () => store.audioRecordingTarget !== undefined,
  set: (value: boolean) => {
    if (!value) store.closeAudioRecording();
  },
});
</script>

<template>
  <SpaceAudioRecorder
    v-model:open="open"
    v-bind="store.audioRecordingTarget ? { spaceId: store.audioRecordingTarget.spaceId } : {}"
    :save="store.saveAudioRecording"
    @failed="toast.error(t('spaces.recording.startFailed'))"
  />
</template>
