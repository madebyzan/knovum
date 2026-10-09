<script setup lang="ts">
import { Play, Plus, Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
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
const roleOptions = computed(() =>
  props.draft.roles.value.map((role) => ({ value: role.key, label: role.key })),
);

function supportsDeliveryInstruction(speakerKey: string) {
  const revisionId = props.draft.roleVoiceRevisionId(speakerKey);
  return Boolean(props.voices.find((voice) => voice.revisionId === revisionId)?.supportsDeliveryInstruction);
}
</script>

<template>
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
</template>
