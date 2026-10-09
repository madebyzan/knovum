<script setup lang="ts">
import { onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoicePreviewSource } from "../../contracts/generated/VoicePreviewSource";
import { useSpeechPreview } from "../../speech/use-speech-preview";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{ source: VoicePreviewSource; disabled: boolean }>();
const emit = defineEmits<{ dirty: [value: boolean] }>();
const { t } = useI18n();
const text = shallowRef("");
const { preview, jobId, listening, error, previewSource, cancel, stopListening, reset } = useSpeechPreview();
watch(() => props.source, reset, { deep: true });
watch(text, (value) => emit("dirty", Boolean(value.trim())));
onScopeDispose(reset);
</script>

<template>
  <section class="space-y-2">
    <UiTextarea
      v-model="text"
      :rows="2"
      :maxlength="500"
      :disabled="disabled || Boolean(jobId)"
      :placeholder="t('spaces.speech.project.previewText')"
      :aria-label="t('spaces.speech.project.previewText')"
    />
    <div class="flex gap-2">
      <UiButton
        size="sm"
        :disabled="disabled || Boolean(jobId) || !text.trim()"
        @click="previewSource(text.trim(), source, {})"
        >{{ t("spaces.speech.project.preview") }}</UiButton
      >
      <UiButton v-if="jobId" size="sm" @click="cancel">{{ t("common.cancel") }}</UiButton>
      <UiButton v-if="listening" size="sm" @click="stopListening">{{
        t("spaces.speech.project.stopListening")
      }}</UiButton>
    </div>
    <SpeechMediaPlayer v-if="preview" kind="audio" :source="preview.source" />
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
  </section>
</template>
