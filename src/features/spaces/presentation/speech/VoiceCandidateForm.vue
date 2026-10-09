<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useVoiceCandidateForm, type VoiceCandidateSourceKind } from "../../speech/use-voice-candidate-form";
import VoiceCandidateModelSelect from "./VoiceCandidateModelSelect.vue";
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
  authorized,
  text,
  candidates,
  activeRunId,
  pendingRequestId,
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
  const voice = await save(candidateId, name, authorized.value);
  if (voice) emit("created", voice);
}
</script>

<template>
  <section class="space-y-4">
    <div class="flex gap-2">
      <UiSelect
        v-if="!sourceNode && !sourceKind"
        v-model="mode"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice.mode')"
        :options="modes"
        :disabled="busy || preparingReference"
      />
      <UiSelect
        v-model="language"
        class="w-40 shrink-0"
        size="sm"
        :label="t('spaces.speech.project.language')"
        :options="languages"
        :disabled="busy || preparingReference"
      />
    </div>
    <VoiceCandidateModelSelect
      v-model="model"
      v-model:available="modelAvailable"
      v-model:reference-transcript="referenceTranscript"
      :source-kind="mode"
      :disabled="busy || preparingReference"
    />
    <template v-if="mode === 'reference'">
      <VoiceReferenceEditor
        v-model="reference"
        :language="language"
        :source-node="props.sourceNode"
        :disabled="busy"
        @busy="preparingReference = $event"
        @dirty="referenceEditorDirty = $event"
      />
      <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.reference.requirements") }}</p>
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
    </template>
    <UiTextarea
      v-else
      v-model="description"
      :rows="2"
      :disabled="busy"
      :placeholder="t('spaces.speech.voice.designDescription')"
      :aria-label="t('spaces.speech.voice.designDescription')"
    />
    <label class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiCheckbox v-model="authorized" :disabled="busy" :label="t('spaces.speech.reference.authorization')" />
      {{ t("spaces.speech.reference.authorization") }}
    </label>
    <div class="space-y-2 border-t border-stroke-divider pt-4">
      <UiTextarea
        v-model="text"
        :rows="3"
        :disabled="busy"
        :placeholder="t('spaces.speech.voice.candidate.text')"
        :aria-label="t('spaces.speech.voice.candidate.text')"
      />
      <div class="flex gap-2">
        <UiButton
          size="sm"
          variant="primary"
          :disabled="busy || !ready || candidates.length >= 16"
          @click="generate"
        >
          {{ t("spaces.speech.voice.candidate.generate") }}
        </UiButton>
        <UiButton v-if="activeRunId || pendingRequestId" size="sm" variant="dangerSubtle" @click="cancel">
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
      :authorized
      @audition="audition"
      @dirty="candidateListDirty = $event"
      @save="saveCandidate"
      @discard="discard"
    />
  </section>
</template>
