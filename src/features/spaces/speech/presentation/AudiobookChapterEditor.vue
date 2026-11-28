<script setup lang="ts">
import { Play, Plus, Trash2 } from "@lucide/vue";
import { computed, nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { ChapterDraft } from "../audiobook-draft";
import type { AudiobookChapterPreparation } from "../use-audiobook-chapter-preparation";
import type { useAudiobookDraft } from "../use-audiobook-draft";
import AudiobookChapterHeader from "./AudiobookChapterHeader.vue";
import SpeechDeliveryInstructionField from "./SpeechDeliveryInstructionField.vue";
import SpeechNarrationPrepareMenu from "./SpeechNarrationPrepareMenu.vue";

const props = defineProps<{
  draft: ReturnType<typeof useAudiobookDraft>;
  voices: readonly VoiceLibraryRevision[];
  preparation: AudiobookChapterPreparation;
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
  <div ref="editorRef" class="space-y-2.5">
    <section
      v-for="(chapter, chapterIndex) in draft.chapters.value"
      :key="chapter.key"
      :data-chapter="chapter.key"
      class="space-y-2 rounded-lg bg-surface-control/75 p-2"
    >
      <AudiobookChapterHeader
        v-model:title="chapter.title"
        :index="chapterIndex"
        :count="draft.chapters.value.length"
        :disabled="disabled || preparation.preparing.value"
        @move="draft.moveChapter(chapterIndex, $event)"
        @remove="draft.removeChapter(chapterIndex)"
      >
        <template #actions>
          <SpeechNarrationPrepareMenu
            compact
            :disabled="
              disabled ||
              (preparation.preparing.value && preparation.preparingChapterKey.value !== chapter.key)
            "
            :empty="!chapter.paragraphs.some((paragraph) => paragraph.text.trim())"
            :preparing="preparation.preparing.value && preparation.preparingChapterKey.value === chapter.key"
            :can-restore="preparation.canRestoreChapter(chapter.key)"
            @prepare="(mode, target) => preparation.prepareChapter(chapter, mode, target)"
            @cancel="preparation.cancel()"
            @restore="preparation.restoreChapter(chapter)"
          />
        </template>
      </AudiobookChapterHeader>
      <div class="space-y-1.5">
        <div
          v-for="(paragraph, paragraphIndex) in chapter.paragraphs"
          :key="paragraph.key"
          class="rounded-md border border-stroke-default bg-surface-raised transition-colors focus-within:border-stroke-focus"
        >
          <div class="flex items-center justify-between gap-1 border-b border-stroke-divider px-1.5 py-1">
            <div class="flex min-w-0 items-center gap-1">
              <span class="px-1 font-mono text-caption-sm text-fg-tertiary tabular-nums">
                #{{ paragraphIndex + 1 }}
              </span>
              <UiSelect
                :model-value="paragraph.speakerKey"
                size="sm"
                class="h-6 w-28 min-w-0 border-transparent bg-transparent px-1.5 text-caption font-medium shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus"
                :label="t('spaces.speech.project.role')"
                :options="roleOptions"
                :disabled="disabled"
                @update:model-value="draft.setParagraphSpeaker(paragraph, $event)"
              />
            </div>
            <div class="flex shrink-0 items-center gap-0.5">
              <span class="px-1 text-caption-sm text-fg-tertiary tabular-nums">
                {{ [...paragraph.text.trim()].length }}
              </span>
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
          </div>
          <UiTextarea
            v-model="paragraph.text"
            data-testid="audiobook-paragraph"
            :rows="2"
            class="border-0 bg-transparent px-2 py-1.5 shadow-none hover:border-0 focus:border-0"
            :aria-label="t('spaces.speech.project.script.label')"
            :maxlength="1000000"
            :disabled="disabled"
            :placeholder="t('spaces.speech.project.script.placeholder')"
          />
          <div v-if="supportsDeliveryInstruction(paragraph.speakerKey)" class="px-2 pb-1.5">
            <SpeechDeliveryInstructionField
              v-model="paragraph.deliveryInstruction"
              :disabled="disabled"
              collapsible
              surface="muted"
            />
          </div>
        </div>
      </div>
      <UiButton
        size="sm"
        variant="ghost"
        class="w-full justify-center text-fg-secondary"
        :disabled="disabled"
        @click="addParagraph(chapter)"
        ><Plus class="size-3.5" />{{ t("spaces.speech.audiobook.addParagraph") }}</UiButton
      >
    </section>
  </div>
</template>
