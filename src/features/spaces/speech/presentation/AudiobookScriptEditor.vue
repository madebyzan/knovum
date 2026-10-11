<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, nextTick, shallowRef, watch, useTemplateRef, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useAudiobookDraft } from "../use-audiobook-draft";
import { voiceRevisionOptions } from "../voice-options";
import AudiobookChapterEditor from "./AudiobookChapterEditor.vue";
import AudiobookMetadataEditor from "./AudiobookMetadataEditor.vue";
import AudiobookOutputSettings from "./AudiobookOutputSettings.vue";
import AudiobookRoleBindings from "./AudiobookRoleBindings.vue";
import AudiobookScriptImport from "./AudiobookScriptImport.vue";
import PronunciationLexiconEditor from "./PronunciationLexiconEditor.vue";
import SpeechEditorPage from "./SpeechEditorPage.vue";
import SpeechEditorTabs from "./SpeechEditorTabs.vue";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";

const props = defineProps<{
  sourceNode: SpaceCanvasNode;
  project: DeepReadonly<SpeechProjectSnapshot>;
  voices: readonly VoiceLibraryRevision[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  preview: [
    text: string,
    voiceRevisionId: string,
    pronunciationLexicon: Record<string, string>,
    deliveryInstruction?: string,
  ];
  busy: [value: boolean];
  draftState: [
    value: boolean,
    input?: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ];
}>();
const { t } = useI18n();
const page = shallowRef("script");
const importing = shallowRef(false);
const editingDisabled = computed(() => props.disabled || importing.value);
watch(importing, (value) => emit("busy", value), { immediate: true });
const chaptersRef = useTemplateRef("chaptersRef");
async function addChapter() {
  if (!draft.roles.value.length) draft.addRole(t("spaces.speech.project.defaultRole"));
  draft.addChapter(t("spaces.speech.audiobook.chapterTitle"));
  await nextTick();
  chaptersRef.value?.focusLastChapter();
}
const pages = computed(() => [
  { value: "script", label: t("spaces.speech.project.script.label") },
  { value: "voices", label: t("spaces.speech.project.roleBindings.label") },
  { value: "settings", label: t("spaces.speech.project.settings") },
]);
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
    draft.needsRevision.value && draft.valid.value ? draft.revisionInput() : undefined,
  ),
);
</script>
<template>
  <SpeechEditorTabs
    v-model="page"
    :items="pages"
    :label="t('spaces.speech.project.editorPages')"
    :invalid-page="
      !draft.lexicon.isValid.value ||
      (draft.output.format.value === 'wav' && Boolean(draft.output.metadata.value.cover))
        ? 'settings'
        : undefined
    "
  >
    <SpeechEditorPage value="script" :active="page">
      <AudiobookScriptImport
        :node="sourceNode"
        :disabled="disabled"
        :initialize="project.project.activeRevision === 1 && !project.segments.length"
        @imported="(script, initialize) => draft.importScript(script, initialize)"
        @busy="importing = $event"
      />
      <p v-if="!draft.chapters.value.length && !importing" class="text-caption text-fg-secondary">
        {{ t("spaces.speech.project.emptyManuscript") }}
      </p>
      <p v-if="draft.configurationError.value" class="text-caption-sm text-fg-danger">
        {{ t("spaces.speech.audiobook.configurationInvalid") }}
      </p>
      <AudiobookChapterEditor
        ref="chaptersRef"
        :draft="draft"
        :voices="voices"
        :disabled="editingDisabled"
        @preview="forwardPreview"
      />
      <UiButton size="sm" variant="ghost" :disabled="editingDisabled" @click="addChapter">
        <Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addChapter") }}
      </UiButton>
    </SpeechEditorPage>
    <SpeechEditorPage value="voices" :active="page">
      <AudiobookRoleBindings
        :roles="draft.roles.value"
        :voice-options="voiceOptions"
        :used-role-keys="draft.usedRoleKeys.value"
        :disabled="editingDisabled"
        @add="draft.addRole(t('spaces.speech.project.defaultRole'))"
        @rename="draft.renameRole"
        @voice="draft.setRoleVoice"
        @remove="draft.removeRole"
      />
    </SpeechEditorPage>
    <SpeechEditorPage value="settings" :active="page">
      <SpeechSettingsDisclosure :title="t('spaces.speech.audiobook.metadataSettings')" :is-expanded="true">
        <AudiobookMetadataEditor v-model="draft.output.metadata.value" :disabled="editingDisabled" />
      </SpeechSettingsDisclosure>
      <SpeechSettingsDisclosure
        :title="t('spaces.speech.audiobook.outputSettings')"
        :is-invalid="draft.output.format.value === 'wav' && Boolean(draft.output.metadata.value.cover)"
      >
        <AudiobookOutputSettings
          v-model:format="draft.output.format.value"
          v-model:sample-rate="draft.output.sampleRate.value"
          v-model:loudness="draft.output.loudness.value"
          v-model:peak="draft.output.peak.value"
          v-model:paragraph-pause="draft.output.paragraphPause.value"
          v-model:chapter-pause="draft.output.chapterPause.value"
          v-model:wav-precision="draft.output.wavPrecision.value"
          :disabled="editingDisabled"
        />
        <p
          v-if="draft.output.format.value === 'wav' && draft.output.metadata.value.cover"
          class="text-caption text-fg-danger"
        >
          {{ t("spaces.speech.audiobook.coverRequiresEncodedOutput") }}
        </p>
      </SpeechSettingsDisclosure>
      <SpeechSettingsDisclosure
        :title="t('spaces.speech.audiobook.lexicon.title')"
        :is-invalid="!draft.lexicon.isValid.value"
      >
        <PronunciationLexiconEditor
          v-model="draft.lexicon.entries.value"
          :valid="draft.lexicon.isValid.value"
          :disabled="editingDisabled"
        />
      </SpeechSettingsDisclosure>
    </SpeechEditorPage>
  </SpeechEditorTabs>
</template>
