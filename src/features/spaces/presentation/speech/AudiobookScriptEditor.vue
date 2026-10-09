<script setup lang="ts">
import { Plus, Save } from "@lucide/vue";
import { computed, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useAudiobookDraft } from "../../speech/use-audiobook-draft";
import { voiceRevisionOptions } from "../../speech/voice-options";
import AudiobookChapterEditor from "./AudiobookChapterEditor.vue";
import AudiobookMetadataEditor from "./AudiobookMetadataEditor.vue";
import AudiobookOutputSettings from "./AudiobookOutputSettings.vue";
import AudiobookRoleBindings from "./AudiobookRoleBindings.vue";
import AudiobookScriptImport from "./AudiobookScriptImport.vue";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";

const props = defineProps<{
  sourceNode: SpaceCanvasNode;
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  save: [input: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">];
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
}>();
const { t } = useI18n();
const draft = useAudiobookDraft(
  () => props.project,
  () => props.voices,
);
const voiceOptions = computed(() => voiceRevisionOptions(props.voices, draft.selectedVoiceRevisionIds.value));
function forwardPreview(
  text: string,
  voiceRevisionId: string,
  pronunciationLexicon: Record<string, string>,
  deliveryInstruction?: string,
) {
  emit("preview", text, voiceRevisionId, pronunciationLexicon, deliveryInstruction);
}
watchEffect(() =>
  emit(
    "draftState",
    draft.dirty.value,
    draft.dirty.value && draft.valid.value ? draft.revisionInput() : undefined,
  ),
);
</script>
<template>
  <div class="space-y-3">
    <AudiobookOutputSettings
      v-model:format="draft.format.value"
      v-model:sample-rate="draft.sampleRate.value"
      v-model:loudness="draft.loudness.value"
      v-model:peak="draft.peak.value"
      v-model:paragraph-pause="draft.paragraphPause.value"
      v-model:chapter-pause="draft.chapterPause.value"
      v-model:wav-precision="draft.wavPrecision.value"
      :disabled="disabled"
    />
    <p v-if="draft.format.value === 'wav' && draft.metadata.value.cover" class="text-caption text-fg-danger">
      {{ t("spaces.speech.audiobook.coverRequiresEncodedOutput") }}
    </p>
    <AudiobookMetadataEditor v-model="draft.metadata.value" :disabled />
    <PronunciationLexiconEditor
      v-model="draft.lexicon.entries.value"
      :valid="draft.lexicon.isValid.value"
      :disabled="disabled"
    />
    <AudiobookScriptImport :node="sourceNode" :disabled="disabled" @imported="draft.importScript" />
    <p v-if="draft.configurationError.value" class="text-danger text-caption-sm">
      {{ t("spaces.speech.audiobook.configurationInvalid") }}
    </p>
    <AudiobookRoleBindings
      :roles="draft.roles.value"
      :voice-options="voiceOptions"
      :used-role-keys="draft.usedRoleKeys.value"
      :disabled="disabled"
      @add="draft.addRole(t('spaces.speech.project.defaultRole'))"
      @rename="draft.renameRole"
      @voice="draft.setRoleVoice"
      @remove="draft.removeRole"
    />
    <AudiobookChapterEditor :draft="draft" :voices="voices" :disabled="disabled" @preview="forwardPreview" />
    <div class="flex gap-2">
      <UiButton
        size="sm"
        :disabled="disabled"
        @click="draft.addChapter(t('spaces.speech.audiobook.chapterTitle'))"
        ><Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addChapter") }}</UiButton
      >
      <UiButton
        size="sm"
        :disabled="disabled || !draft.dirty.value || !draft.valid.value"
        @click="emit('save', draft.revisionInput())"
        ><Save class="size-3.5" />{{ t("spaces.speech.project.saveRevision") }}</UiButton
      >
    </div>
  </div>
</template>
