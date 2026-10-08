<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PreparedVoiceReference } from "../../contracts/generated/PreparedVoiceReference";
import {
  useVoiceReferenceEditor,
  type VoiceReferenceEditorProps,
} from "../../speech/use-voice-reference-editor";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<VoiceReferenceEditorProps>();
const model = defineModel<PreparedVoiceReference | undefined>();
const emit = defineEmits<{ busy: [value: boolean] }>();
const { t } = useI18n();
const player = useTemplateRef("player");
const {
  path,
  start,
  end,
  duration,
  sourceVersion,
  jobId,
  error,
  selectionValid,
  canPrepare,
  cancel,
  choose,
  prepare,
  selectionLoaded,
} = useVoiceReferenceEditor(props, model, (value) => emit("busy", value));
</script>

<template>
  <section class="space-y-2">
    <UiButton
      v-if="!sourceNode"
      data-testid="reference-choose"
      size="sm"
      :disabled="disabled || Boolean(jobId)"
      @click="choose"
      >{{ t("spaces.speech.reference.choose") }}</UiButton
    >
    <template v-if="path">
      <SpeechMediaPlayer
        :key="sourceVersion"
        ref="player"
        :kind="sourceNode?.data.kind === 'video' ? 'video' : 'audio'"
        :source="convertFileSrc(path)"
        @loaded="selectionLoaded"
      />
      <p
        v-if="duration !== undefined && duration < 5000"
        class="text-caption text-fg-secondary"
        role="status"
      >
        {{ t("spaces.speech.reference.tooShort") }}
      </p>
      <UiButton
        size="sm"
        :disabled="!selectionValid"
        @click="player?.seekTo(Number(start) * 1000, Number(end) * 1000)"
        >{{ t("spaces.speech.reference.play") }}</UiButton
      >
      <div class="grid grid-cols-2 gap-2">
        <label class="text-caption text-fg-secondary"
          >{{ t("spaces.speech.reference.start")
          }}<UiInput
            v-model="start"
            data-testid="reference-start"
            size="sm"
            type="number"
            min="0"
            step="0.1"
            :disabled="disabled || Boolean(jobId)"
        /></label>
        <label class="text-caption text-fg-secondary"
          >{{ t("spaces.speech.reference.end")
          }}<UiInput
            v-model="end"
            data-testid="reference-end"
            size="sm"
            type="number"
            min="0"
            step="0.1"
            :disabled="disabled || Boolean(jobId)"
        /></label>
      </div>
      <UiButton data-testid="reference-prepare" size="sm" :disabled="!canPrepare" @click="prepare">{{
        t("spaces.speech.reference.prepare")
      }}</UiButton>
      <UiButton v-if="jobId" data-testid="reference-cancel" size="sm" @click="cancel">{{
        t("common.cancel")
      }}</UiButton>
    </template>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
