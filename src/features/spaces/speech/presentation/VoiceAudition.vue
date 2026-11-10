<script setup lang="ts">
import { computed, onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoicePreviewSource } from "../../contracts/generated/VoicePreviewSource";
import { useSpeechPreview } from "../use-speech-preview";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{ source: VoicePreviewSource; disabled: boolean }>();
const emit = defineEmits<{ dirty: [value: boolean] }>();
const { t } = useI18n();
const text = shallowRef("");
const { preview, progress, jobId, listening, error, previewSource, cancel, stopListening, reset } =
  useSpeechPreview();
watch(() => props.source, reset, { deep: true });
watch(text, (value) => emit("dirty", Boolean(value.trim())));
onScopeDispose(reset);

const progressLabel = computed(() => {
  if (progress.value?.stage === "preparing") return t("spaces.speech.preview.preparing");
  if (progress.value?.stage === "synthesizing") return t("spaces.speech.preview.synthesizing");
  return t("common.loading");
});
</script>

<template>
  <section class="space-y-3">
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.preview.text.label") }}</span>
      <UiTextarea
        v-model="text"
        :rows="2"
        :maxlength="500"
        :disabled="disabled || Boolean(jobId)"
        :placeholder="t('spaces.speech.preview.text.placeholder')"
        :aria-label="t('spaces.speech.preview.text.placeholder')"
      />
    </label>
    <div v-if="jobId" class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiSpinner size="sm" />
      <span>{{ progressLabel }}</span>
    </div>
    <SpeechMediaPlayer v-if="preview" kind="audio" :source="preview.source" @play="stopListening" />
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <div class="flex items-center justify-between gap-2 border-t border-stroke-divider pt-3">
      <div class="flex items-center gap-2">
        <UiButton
          size="sm"
          :disabled="disabled || Boolean(jobId) || !text.trim()"
          @click="previewSource(text.trim(), source, {})"
        >
          <UiSpinner v-if="Boolean(jobId)" size="sm" />
          {{ Boolean(jobId) ? t("spaces.speech.preview.synthesizing") : t("spaces.speech.preview.action") }}
        </UiButton>
        <UiButton v-if="jobId" size="sm" @click="cancel">{{ t("common.cancel") }}</UiButton>
        <UiButton v-if="listening" size="sm" @click="stopListening">{{
          t("spaces.speech.preview.stop")
        }}</UiButton>
      </div>
      <slot name="actions" />
    </div>
  </section>
</template>
