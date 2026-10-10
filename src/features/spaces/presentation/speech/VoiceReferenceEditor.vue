<script setup lang="ts">
import { Check, Play, Square } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onBeforeUnmount, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { PreparedVoiceReference } from "../../contracts/generated/PreparedVoiceReference";
import {
  useVoiceReferenceEditor,
  type VoiceReferenceEditorProps,
} from "../../speech/use-voice-reference-editor";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import VoiceReferenceSelection from "./VoiceReferenceSelection.vue";

const props = defineProps<VoiceReferenceEditorProps>();
const model = defineModel<PreparedVoiceReference | undefined>();
const emit = defineEmits<{ busy: [value: boolean]; dirty: [value: boolean] }>();
const { t } = useI18n();
const player = useTemplateRef("player");
const {
  path,
  start,
  end,
  durationMs,
  sourceVersion,
  jobId,
  isCancelling,
  error,
  selectionValid,
  canPrepare,
  cancel,
  choose,
  prepare,
  selectionLoaded,
} = useVoiceReferenceEditor(props, model);
const dirty = computed(() => !props.sourceNode && Boolean(path.value));
watch(dirty, (value) => emit("dirty", value));
onBeforeUnmount(() => emit("busy", false));
const editingDisabled = computed(() => props.disabled || Boolean(jobId.value));
async function prepareReference() {
  emit("busy", true);
  try {
    await prepare();
  } finally {
    emit("busy", false);
  }
}
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
        :preview="false"
        @loaded="selectionLoaded"
      />
      <VoiceReferenceSelection
        v-model:start="start"
        v-model:end="end"
        :duration-ms="durationMs"
        :disabled="editingDisabled"
        :valid="selectionValid"
      />
      <div class="flex flex-wrap items-center gap-2">
        <UiButton
          size="sm"
          :disabled="!selectionValid || editingDisabled"
          @click="player?.seekTo(Number(start) * 1000, Number(end) * 1000)"
        >
          <Play class="size-3.5" />{{ t("spaces.speech.reference.play") }}
        </UiButton>
        <UiButton
          v-if="jobId"
          data-testid="reference-cancel"
          size="sm"
          :disabled="isCancelling"
          @click="cancel"
        >
          <Square class="size-3.5" />{{ t("common.cancel") }}
        </UiButton>
        <UiButton
          v-else
          data-testid="reference-prepare"
          size="sm"
          :disabled="!canPrepare"
          @click="prepareReference"
        >
          <Check v-if="model" class="size-3.5 text-fg-accent" />{{
            t(model ? "spaces.speech.reference.prepared" : "spaces.speech.reference.prepare")
          }}
        </UiButton>
      </div>
    </template>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
