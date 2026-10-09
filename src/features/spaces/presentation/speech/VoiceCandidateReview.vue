<script setup lang="ts">
import { shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PlayableVoiceCandidate } from "../../speech/voice-candidate-selection";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

defineProps<{ candidate: DeepReadonly<PlayableVoiceCandidate>; disabled: boolean; authorized: boolean }>();
const emit = defineEmits<{ audition: []; dirty: [value: boolean]; save: [name: string]; discard: [] }>();
const name = shallowRef("");
const { t } = useI18n();
watch(name, (value) => emit("dirty", Boolean(value.trim())));
</script>

<template>
  <section class="grid gap-2 py-3 md:grid-cols-[minmax(0,1fr)_minmax(12rem,0.7fr)] md:items-center">
    <SpeechMediaPlayer kind="audio" :source="candidate.source" @ended="emit('audition')" />
    <div class="space-y-2">
      <p v-if="candidate.auditionedAt === null" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.voice.candidate.auditionRequired") }}
      </p>
      <UiInput
        v-model="name"
        size="sm"
        :disabled
        :placeholder="t('spaces.speech.voice.name')"
        :aria-label="t('spaces.speech.voice.name')"
      />
      <div class="flex gap-2">
        <UiButton
          size="sm"
          variant="primary"
          :disabled="disabled || !name.trim() || !authorized || candidate.auditionedAt === null"
          @click="emit('save', name.trim())"
        >
          {{ t("spaces.speech.voice.candidate.save") }}
        </UiButton>
        <UiButton size="sm" :disabled @click="emit('discard')">
          {{ t("spaces.speech.voice.candidate.discard") }}
        </UiButton>
      </div>
    </div>
  </section>
</template>
