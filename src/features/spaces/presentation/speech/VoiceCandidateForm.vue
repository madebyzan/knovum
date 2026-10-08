<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useVoiceCandidateForm } from "../../speech/use-voice-candidate-form";
import VoiceCandidateModelSelect from "./VoiceCandidateModelSelect.vue";
import VoiceCandidateReview from "./VoiceCandidateReview.vue";
import VoiceReferenceEditor from "./VoiceReferenceEditor.vue";

const emit = defineEmits<{ created: [voice: VoiceLibraryRevision] }>();
const props = defineProps<{ sourceNode?: SpaceCanvasNode | undefined; spaceId: string }>();
const {
  t,
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
  error,
  modes,
  languages,
  ready,
  generate,
  save,
  cancel,
  discard,
  audition,
} = useVoiceCandidateForm(
  () => props.spaceId,
  (voice) => emit("created", voice),
);
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2">
    <div class="flex gap-2">
      <UiSelect
        v-if="!sourceNode"
        v-model="mode"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voiceMode')"
        :options="modes"
        :disabled="busy || preparingReference"
      />
      <UiSelect
        v-model="language"
        size="sm"
        :label="t('spaces.speech.language')"
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
        :space-id="spaceId"
        :language="language"
        :source-node="props.sourceNode"
        :disabled="busy"
        @busy="preparingReference = $event"
      />
      <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.referenceRequirements") }}</p>
      <UiTextarea
        v-if="referenceTranscript === 'optionalConfirmed'"
        v-model="transcript"
        :rows="2"
        :disabled="busy || preparingReference"
        :placeholder="t('spaces.speech.referenceTranscript')"
        :aria-label="t('spaces.speech.referenceTranscript')"
        @update:model-value="transcriptConfirmed = false"
      />
      <label
        v-if="referenceTranscript === 'optionalConfirmed' && transcript.trim()"
        class="flex items-center gap-2 text-caption text-fg-secondary"
      >
        <UiCheckbox
          v-model="transcriptConfirmed"
          :disabled="busy || preparingReference"
          :label="t('spaces.speech.transcriptConfirmed')"
        />
        {{ t("spaces.speech.transcriptConfirmed") }}
      </label>
    </template>
    <UiTextarea
      v-else
      v-model="description"
      :rows="2"
      :disabled="busy"
      :placeholder="t('spaces.speech.designDescription')"
      :aria-label="t('spaces.speech.designDescription')"
    />
    <UiTextarea
      v-model="text"
      :rows="3"
      :disabled="busy"
      :placeholder="t('spaces.speech.candidateText')"
      :aria-label="t('spaces.speech.candidateText')"
    />
    <label class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiCheckbox v-model="authorized" :disabled="busy" :label="t('spaces.speech.voiceAuthorization')" />
      {{ t("spaces.speech.voiceAuthorization") }}
    </label>
    <div class="flex gap-2">
      <UiButton size="sm" :disabled="busy || !ready || candidates.length >= 16" @click="generate">{{
        t("spaces.speech.generateCandidate")
      }}</UiButton>
      <UiButton v-if="activeRunId || pendingRequestId" size="sm" variant="dangerSubtle" @click="cancel">{{
        t("spaces.speech.cancel")
      }}</UiButton>
    </div>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
    <VoiceCandidateReview
      v-for="candidate in candidates"
      :key="candidate.candidateId"
      :candidate="candidate"
      :disabled="busy"
      :authorized="authorized"
      @audition="audition(candidate.candidateId)"
      @save="save(candidate.candidateId, $event, authorized)"
      @discard="discard(candidate.candidateId)"
    />
  </section>
</template>
