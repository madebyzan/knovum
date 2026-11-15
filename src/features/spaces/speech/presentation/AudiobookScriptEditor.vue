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
const draft = useAudiobookDraft(
  () => props.project,
  () => props.voices,
);
const voiceOptions = computed(() => voiceRevisionOptions(props.voices, draft.selectedVoiceRevisionIds.value));
const roleUsageCounts = computed(() => {
  const counts = new Map<string, number>();
  for (const chapter of draft.chapters.value) {
    for (const paragraph of chapter.paragraphs) {
      counts.set(paragraph.speakerKey, (counts.get(paragraph.speakerKey) ?? 0) + 1);
    }
  }
  return counts;
});
const paragraphCount = computed(() =>
  draft.chapters.value.reduce((total, chapter) => total + chapter.paragraphs.length, 0),
);
const rolesInvalid = computed(() => {
  const validIds = new Set(props.voices.map((voice) => voice.revisionId));
  return draft.roles.value.some((role) => !validIds.has(role.voiceRevisionId));
});
const settingsInvalid = computed(
  () =>
    !draft.lexicon.isValid.value ||
    (draft.output.format.value === "wav" && Boolean(draft.output.metadata.value.cover)),
);
const pages = computed(() => [
  {
    value: "script",
    label: t("spaces.speech.project.script.label"),
    badge: draft.chapters.value.length || undefined,
  },
  {
    value: "voices",
    label: t("spaces.speech.project.roleBindings.label"),
    badge: draft.roles.value.length || undefined,
    invalid: rolesInvalid.value,
  },
  {
    value: "settings",
    label: t("spaces.speech.project.settings"),
    invalid: settingsInvalid.value,
  },
]);
const outputSummary = computed(
  () => `${draft.output.format.value.toUpperCase()} · ${Number(draft.output.sampleRate.value) / 1000} kHz`,
);
const lexiconSummary = computed(() =>
  draft.lexicon.entries.value.length ? String(draft.lexicon.entries.value.length) : undefined,
);
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
    :invalid-page="settingsInvalid ? 'settings' : undefined"
  >
    <SpeechEditorPage value="script" :active="page">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <AudiobookScriptImport
          :node="sourceNode"
          :disabled="disabled"
          :initialize="project.project.activeRevision === 1 && !project.segments.length"
          @imported="draft.importScript($event)"
          @initialized="draft.initializeFromScript($event)"
          @busy="importing = $event"
        />
        <span v-if="draft.chapters.value.length" class="text-caption-sm text-fg-tertiary tabular-nums">
          {{ draft.chapters.value.length }} · {{ paragraphCount }}
        </span>
      </div>
      <div
        v-if="!draft.chapters.value.length && !importing"
        class="rounded-panel border border-dashed border-stroke-default px-3 py-5 text-center text-caption text-fg-secondary"
      >
        {{ t("spaces.speech.project.emptyManuscript") }}
      </div>
      <p
        v-if="draft.configurationError.value"
        class="rounded-panel bg-surface-danger-subtle px-3 py-2 text-caption-sm text-fg-danger"
      >
        {{ t("spaces.speech.audiobook.configurationInvalid") }}
      </p>
      <AudiobookChapterEditor
        ref="chaptersRef"
        :draft="draft"
        :voices="voices"
        :disabled="editingDisabled"
        @preview="forwardPreview"
      />
      <UiButton size="sm" variant="secondary" :disabled="editingDisabled" @click="addChapter">
        <Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addChapter") }}
      </UiButton>
    </SpeechEditorPage>
    <SpeechEditorPage value="voices" :active="page">
      <AudiobookRoleBindings
        :roles="draft.roles.value"
        :voice-options="voiceOptions"
        :used-role-keys="draft.usedRoleKeys.value"
        :role-usage-counts="roleUsageCounts"
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
        :summary="outputSummary"
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
        :summary="lexiconSummary"
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
