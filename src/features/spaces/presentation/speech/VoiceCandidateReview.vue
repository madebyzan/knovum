<script setup lang="ts">
import { shallowRef, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PlayableVoiceCandidate } from "../../speech/voice-candidate-selection";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

defineProps<{ candidate: DeepReadonly<PlayableVoiceCandidate>; disabled: boolean; authorized: boolean }>();
const emit = defineEmits<{ audition: []; save: [name: string]; discard: [] }>();
const name = shallowRef("");
const { t } = useI18n();
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <SpeechMediaPlayer kind="audio" :source="candidate.source" @ended="emit('audition')" />
    <p v-if="candidate.auditionedAt === null" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.candidateAuditionRequired") }}
    </p>
    <UiInput
      v-model="name"
      size="sm"
      :disabled
      :placeholder="t('spaces.speech.voiceName')"
      :aria-label="t('spaces.speech.voiceName')"
    />
    <div class="flex gap-2">
      <UiButton
        size="sm"
        variant="primary"
        :disabled="disabled || !name.trim() || !authorized || candidate.auditionedAt === null"
        @click="emit('save', name.trim())"
        >{{ t("spaces.speech.saveCandidate") }}</UiButton
      >
      <UiButton size="sm" :disabled @click="emit('discard')">{{
        t("spaces.speech.discardCandidate")
      }}</UiButton>
    </div>
  </section>
</template>
