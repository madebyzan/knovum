<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useVoiceCandidateForm, type VoiceCandidateSourceKind } from "../../speech/use-voice-candidate-form";
import VoiceCandidateOptions from "./VoiceCandidateOptions.vue";
import VoiceCandidateReviews from "./VoiceCandidateReviews.vue";
import VoiceReferenceEditor from "./VoiceReferenceEditor.vue";

const emit = defineEmits<{ created: [voice: VoiceLibraryRevision]; dirty: [value: boolean] }>();
const props = defineProps<{
  sourceNode?: SpaceCanvasNode | undefined;
  sourceKind?: VoiceCandidateSourceKind | undefined;
}>();
const { t } = useI18n();
const {
  mode,
  model,
  modelAvailable,
  referenceTranscript,
  language,
  description,
  reference,
  preparingReference,
  transcript,
  transcriptConfirmed,
  text,
  candidates,
  activeRunId,
  pendingRequestId,
  isCancelling,
  busy,
  errors,
  modes,
  languages,
  ready,
  dirty: draftDirty,
  generate,
  save,
  cancel,
  discard,
  audition,
} = useVoiceCandidateForm(() => (props.sourceNode ? "reference" : props.sourceKind));
const referenceEditorDirty = shallowRef(false);
const candidateListDirty = shallowRef(false);
const dirty = computed(() => draftDirty.value || referenceEditorDirty.value || candidateListDirty.value);
watch(dirty, (value) => emit("dirty", value), { immediate: true });

async function saveCandidate(candidateId: string, name: string) {
  const voice = await save(candidateId, name);
  if (voice) emit("created", voice);
}
</script>

<template>
  <section class="space-y-4">
    <VoiceCandidateOptions
      v-model:mode="mode"
      v-model:model="model"
      v-model:available="modelAvailable"
      v-model:reference-transcript="referenceTranscript"
      v-model:language="language"
      :fixed-source="Boolean(sourceNode || sourceKind)"
      :disabled="busy || preparingReference"
      :modes="modes"
      :languages="languages"
    />
    <section v-if="mode === 'reference'" class="space-y-3 border-t border-stroke-divider pt-3">
      <h3 class="text-caption font-medium text-fg-primary">{{ t("spaces.speech.reference.title") }}</h3>
      <VoiceReferenceEditor
        v-model="reference"
        :language="language"
        :source-node="props.sourceNode"
        :disabled="busy"
        @busy="preparingReference = $event"
        @dirty="referenceEditorDirty = $event"
      />
      <p class="text-caption-sm leading-relaxed text-fg-tertiary">
        {{ t("spaces.speech.reference.requirements") }}
      </p>
      <UiTextarea
        v-if="referenceTranscript === 'optionalConfirmed'"
        v-model="transcript"
        :rows="2"
        :disabled="busy || preparingReference"
        :placeholder="t('spaces.speech.reference.transcript')"
        :aria-label="t('spaces.speech.reference.transcript')"
        @update:model-value="transcriptConfirmed = false"
      />
      <label
        v-if="referenceTranscript === 'optionalConfirmed' && transcript.trim()"
        class="flex items-center gap-2 text-caption text-fg-secondary"
      >
        <UiCheckbox
          v-model="transcriptConfirmed"
          :disabled="busy || preparingReference"
          :label="t('spaces.speech.reference.transcriptConfirmed')"
        />
        {{ t("spaces.speech.reference.transcriptConfirmed") }}
      </label>
    </section>
    <UiTextarea
      v-else
      v-model="description"
      :rows="2"
      :disabled="busy"
      :placeholder="t('spaces.speech.voice.design.placeholder')"
      :aria-label="t('spaces.speech.voice.design.placeholder')"
    />
    <div class="space-y-3 border-t border-stroke-divider pt-3">
      <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
        <span>{{ t("spaces.speech.preview.text.label") }}</span>
        <UiTextarea
          v-model="text"
          :rows="2"
          :disabled="busy"
          :placeholder="t('spaces.speech.preview.text.label')"
          :aria-label="t('spaces.speech.preview.text.label')"
        />
      </label>
      <div class="flex items-center justify-end gap-2">
        <UiButton
          size="sm"
          variant="primary"
          :disabled="busy || !ready || candidates.length >= 16"
          @click="generate"
        >
          {{ t(busy ? "spaces.speech.project.starting" : "spaces.speech.voice.candidate.generate") }}
        </UiButton>
        <UiButton
          v-if="activeRunId || pendingRequestId"
          size="sm"
          variant="dangerSubtle"
          :disabled="isCancelling"
          @click="cancel"
        >
          {{ t("common.cancel") }}
        </UiButton>
      </div>
    </div>
    <div v-if="errors.length" role="alert" class="space-y-1">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <VoiceCandidateReviews
      v-if="candidates.length"
      :candidates
      :disabled="busy"
      @audition="audition"
      @dirty="candidateListDirty = $event"
      @save="saveCandidate"
      @discard="discard"
    />
  </section>
</template>
