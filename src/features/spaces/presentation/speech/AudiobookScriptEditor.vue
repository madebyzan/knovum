<script setup lang="ts">
import { Play, Plus, Save, Trash2 } from "@lucide/vue";
import { computed, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useAudiobookDraft } from "../../speech/use-audiobook-draft";
import { voiceRevisionOptions } from "../../speech/voice-options";
import AudiobookChapterHeader from "./AudiobookChapterHeader.vue";
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
  dirty: [value: boolean];
}>();
const { t } = useI18n();
const draft = useAudiobookDraft(
  () => props.project,
  () => props.voices,
);
const voiceOptions = computed(() => voiceRevisionOptions(props.voices, draft.selectedVoiceRevisionIds.value));
const roleOptions = computed(() => draft.roles.value.map((role) => ({ value: role.key, label: role.key })));
function supportsDeliveryInstruction(speakerKey: string) {
  const revisionId = draft.roleVoiceRevisionId(speakerKey);
  return Boolean(props.voices.find((voice) => voice.revisionId === revisionId)?.supportsDeliveryInstruction);
}
watch(draft.dirty, (value) => emit("dirty", value), { immediate: true });
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
    <section
      v-for="(chapter, chapterIndex) in draft.chapters.value"
      :key="chapter.key"
      class="space-y-2 rounded-md border border-stroke-divider p-2"
    >
      <AudiobookChapterHeader
        v-model:title="chapter.title"
        :index="chapterIndex"
        :count="draft.chapters.value.length"
        :disabled="disabled"
        @move="draft.moveChapter(chapterIndex, $event)"
        @remove="draft.removeChapter(chapterIndex)"
      />
      <div
        v-for="(paragraph, paragraphIndex) in chapter.paragraphs"
        :key="paragraph.key"
        class="space-y-1 border-t border-stroke-divider pt-2"
      >
        <UiTextarea
          v-model="paragraph.text"
          :rows="3"
          :maxlength="1000000"
          :disabled="disabled"
          :placeholder="t('spaces.speech.project.scriptPlaceholder')"
        />
        <div class="flex items-center gap-1">
          <UiSelect
            :model-value="paragraph.speakerKey"
            size="sm"
            class="min-w-0 flex-1"
            :label="t('spaces.speech.project.role')"
            :options="roleOptions"
            :disabled="disabled"
            @update:model-value="draft.setParagraphSpeaker(paragraph, $event)"
          />
          <UiButton
            size="sm"
            variant="ghost"
            :aria-label="t('spaces.speech.project.preview')"
            :disabled="
              disabled ||
              !draft.lexicon.isValid.value ||
              !paragraph.text.trim() ||
              !draft.roleVoiceRevisionId(paragraph.speakerKey)
            "
            @click="
              emit(
                'preview',
                [...paragraph.text.trim()].slice(0, 500).join(''),
                draft.roleVoiceRevisionId(paragraph.speakerKey),
                draft.lexicon.serializeLexicon(),
                paragraph.deliveryInstruction.trim() || undefined,
              )
            "
            ><Play class="size-3.5"
          /></UiButton>
          <UiButton
            size="sm"
            variant="dangerSubtle"
            :aria-label="t('spaces.speech.audiobook.removeParagraph')"
            :disabled="disabled"
            @click="draft.removeParagraph(chapter, paragraphIndex)"
            ><Trash2 class="size-3.5"
          /></UiButton>
        </div>
        <UiTextarea
          v-if="supportsDeliveryInstruction(paragraph.speakerKey)"
          v-model="paragraph.deliveryInstruction"
          :rows="2"
          :maxlength="500"
          :disabled="disabled"
          :placeholder="t('spaces.speech.project.deliveryInstructionPlaceholder')"
          :aria-label="t('spaces.speech.project.deliveryInstruction')"
        />
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled" @click="draft.addParagraph(chapter)"
        ><Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addParagraph") }}</UiButton
      >
    </section>
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
