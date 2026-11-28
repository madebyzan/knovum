<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, nextTick, shallowRef, watch, useTemplateRef, watchEffect, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SaveSpeechProjectRevisionRequest } from "../../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { formatEstimatedDuration } from "../spoken-duration";
import { useAudiobookChapterPreparation } from "../use-audiobook-chapter-preparation";
import { useAudiobookDraft } from "../use-audiobook-draft";
import { voiceRevisionOptions } from "../voice-options";
import AudiobookChapterEditor from "./AudiobookChapterEditor.vue";
import AudiobookProductionSettings from "./AudiobookProductionSettings.vue";
import AudiobookRoleBindings from "./AudiobookRoleBindings.vue";
import AudiobookScriptImport from "./AudiobookScriptImport.vue";
import SpeechEditorPage from "./SpeechEditorPage.vue";
import SpeechEditorTabs from "./SpeechEditorTabs.vue";
import SpeechNarrationPrepareMenu from "./SpeechNarrationPrepareMenu.vue";

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
const chapterPrep = useAudiobookChapterPreparation(() => draft.roles.value[0]?.key ?? "");
const editingDisabled = computed(() => props.disabled || importing.value || chapterPrep.preparing.value);
watch([importing, chapterPrep.preparing], ([imp, prep]) => emit("busy", imp || prep), { immediate: true });
const hasAnyChapterText = computed(() =>
  draft.chapters.value.some((chapter) => chapter.paragraphs.some((paragraph) => paragraph.text.trim())),
);
const batchProgressLabel = computed(() =>
  chapterPrep.batchProgress.value
    ? `${chapterPrep.batchProgress.value.current}/${chapterPrep.batchProgress.value.total}`
    : undefined,
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
const estimatedDuration = computed(() =>
  formatEstimatedDuration(draft.chapters.value.flatMap((c) => c.paragraphs.map((p) => p.text)).join(" ")),
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
        <div class="flex flex-wrap items-center gap-1.5">
          <AudiobookScriptImport
            :node="sourceNode"
            :disabled="editingDisabled"
            :initialize="project.project.activeRevision === 1 && !project.segments.length"
            @imported="draft.importScript($event)"
            @initialized="draft.initializeFromScript($event)"
            @busy="importing = $event"
          />
          <SpeechNarrationPrepareMenu
            v-if="draft.chapters.value.length"
            :label="
              draft.chapters.value.length > 1
                ? t('spaces.speech.project.prepareNarration.actionAll')
                : t('spaces.speech.project.prepareNarration.action')
            "
            :restore-label="t('spaces.speech.project.prepareNarration.restoreAll')"
            :progress-label="batchProgressLabel"
            :disabled="editingDisabled && chapterPrep.preparingScope.value !== 'book'"
            :empty="!hasAnyChapterText"
            :preparing="chapterPrep.preparing.value && chapterPrep.preparingScope.value === 'book'"
            :can-restore="chapterPrep.canRestoreBook.value"
            @prepare="(mode, target) => chapterPrep.prepareAllChapters(draft.chapters.value, mode, target)"
            @cancel="chapterPrep.cancel()"
            @restore="chapterPrep.restoreAllChapters(draft.chapters.value)"
          />
        </div>
        <span v-if="draft.chapters.value.length" class="text-caption-sm text-fg-tertiary tabular-nums">
          {{ draft.chapters.value.length }} · {{ paragraphCount
          }}<template v-if="estimatedDuration"> · {{ estimatedDuration }}</template>
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
        :preparation="chapterPrep"
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
      <AudiobookProductionSettings :draft="draft" :disabled="editingDisabled" />
    </SpeechEditorPage>
  </SpeechEditorTabs>
</template>
