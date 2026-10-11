<script setup lang="ts">
import { Play, Plus, Trash2 } from "@lucide/vue";
import { computed, nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { ChapterDraft } from "../../speech/audiobook-draft";
import type { useAudiobookDraft } from "../../speech/use-audiobook-draft";
import AudiobookChapterHeader from "./AudiobookChapterHeader.vue";

const props = defineProps<{
  draft: ReturnType<typeof useAudiobookDraft>;
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
}>();
const { t } = useI18n();
const editorRef = useTemplateRef<HTMLElement>("editorRef");
async function addParagraph(chapter: ChapterDraft) {
  props.draft.addParagraph(chapter);
  await nextTick();
  const section = editorRef.value?.querySelector(`[data-chapter="${CSS.escape(chapter.key)}"]`);
  const fields = section?.querySelectorAll<HTMLTextAreaElement>('[data-testid="audiobook-paragraph"]');
  fields?.[fields.length - 1]?.focus();
}
function focusLastChapter() {
  const fields = editorRef.value?.querySelectorAll<HTMLInputElement>("input");
  const field = fields?.[fields.length - 1];
  field?.focus();
  field?.select();
}
defineExpose({ focusLastChapter });
const roleOptions = computed(() =>
  props.draft.roles.value.map((role) => ({ value: role.key, label: role.key })),
);

function supportsDeliveryInstruction(speakerKey: string) {
  const revisionId = props.draft.roleVoiceRevisionId(speakerKey);
  return Boolean(props.voices.find((voice) => voice.revisionId === revisionId)?.supportsDeliveryInstruction);
}
</script>

<template>
  <div ref="editorRef" class="space-y-3">
    <section
      v-for="(chapter, chapterIndex) in draft.chapters.value"
      :key="chapter.key"
      :data-chapter="chapter.key"
      class="space-y-2 border-t border-stroke-divider pt-2"
    >
      <AudiobookChapterHeader
        v-model:title="chapter.title"
        :index="chapterIndex"
        :count="draft.chapters.value.length"
        :disabled="disabled"
        @move="draft.moveChapter(chapterIndex, $event)"
        @remove="draft.removeChapter(chapterIndex)"
      />
      <div v-for="(paragraph, paragraphIndex) in chapter.paragraphs" :key="paragraph.key" class="space-y-1.5">
        <UiTextarea
          v-model="paragraph.text"
          data-testid="audiobook-paragraph"
          :rows="4"
          :aria-label="t('spaces.speech.project.script.label')"
          :maxlength="1000000"
          :disabled="disabled"
          :placeholder="t('spaces.speech.project.script.placeholder')"
        />
        <div class="flex items-center gap-1">
          <UiSelect
            :model-value="paragraph.speakerKey"
            size="sm"
            class="w-32 min-w-0"
            :label="t('spaces.speech.project.role')"
            :options="roleOptions"
            :disabled="disabled"
            @update:model-value="draft.setParagraphSpeaker(paragraph, $event)"
          />
          <UiIconButton
            size="sm"
            tooltip
            :label="t('spaces.speech.preview.action')"
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
          /></UiIconButton>
          <UiIconButton
            size="sm"
            tooltip
            class="hover:text-fg-danger"
            :label="t('spaces.speech.audiobook.removeParagraph')"
            :disabled="disabled"
            @click="draft.removeParagraph(chapter, paragraphIndex)"
            ><Trash2 class="size-3.5"
          /></UiIconButton>
        </div>
        <UiTextarea
          v-if="supportsDeliveryInstruction(paragraph.speakerKey)"
          v-model="paragraph.deliveryInstruction"
          :rows="2"
          :maxlength="500"
          :disabled="disabled"
          :placeholder="t('spaces.speech.project.deliveryInstruction.placeholder')"
          :aria-label="t('spaces.speech.project.deliveryInstruction.label')"
        />
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled" @click="addParagraph(chapter)"
        ><Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addParagraph") }}</UiButton
      >
    </section>
  </div>
</template>
