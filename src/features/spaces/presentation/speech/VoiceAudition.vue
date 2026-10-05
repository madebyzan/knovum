<script setup lang="ts">
import { onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoicePreviewSource } from "../../contracts/generated/VoicePreviewSource";
import { useSpeechPreview } from "../../speech/use-speech-preview";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{ source: VoicePreviewSource; disabled: boolean }>();
const { t } = useI18n();
const text = shallowRef("");
const error = shallowRef<string>();
const preview = useSpeechPreview((message) => {
  error.value = message;
});
watch(() => props.source, preview.reset, { deep: true });
onScopeDispose(preview.reset);
</script>

<template>
  <section class="space-y-2">
    <UiTextarea
      v-model="text"
      :rows="2"
      :maxlength="500"
      :disabled="disabled || Boolean(preview.jobId.value)"
      :placeholder="t('spaces.speech.previewText')"
    />
    <div class="flex gap-2">
      <UiButton
        size="sm"
        :disabled="disabled || Boolean(preview.jobId.value) || !text.trim()"
        @click="preview.previewSource(text.trim(), source, {})"
        >{{ t("spaces.speech.preview") }}</UiButton
      >
      <UiButton v-if="preview.jobId.value" size="sm" @click="preview.cancel">{{
        t("spaces.speech.cancel")
      }}</UiButton>
      <UiButton v-if="preview.listening.value" size="sm" @click="preview.stopListening">{{
        t("spaces.speech.stopListening")
      }}</UiButton>
    </div>
    <SpeechMediaPlayer v-if="preview.preview.value" kind="audio" :source="preview.preview.value.source" />
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
  </section>
</template>
