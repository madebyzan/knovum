<script setup lang="ts">
import { shallowRef, watch, type DeepReadonly } from "vue";

import type { PlayableVoiceCandidate } from "../voice-candidate-selection";
import VoiceCandidateReview from "./VoiceCandidateReview.vue";

const props = defineProps<{
  candidates: readonly DeepReadonly<PlayableVoiceCandidate>[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  audition: [candidateId: string];
  dirty: [value: boolean];
  save: [candidateId: string, name: string];
  discard: [candidateId: string];
}>();
const namedCandidateIds = shallowRef<ReadonlySet<string>>(new Set());

watch(
  () => props.candidates,
  (items) => {
    const available = new Set(items.map(({ candidateId }) => candidateId));
    namedCandidateIds.value = new Set([...namedCandidateIds.value].filter((id) => available.has(id)));
    emit("dirty", namedCandidateIds.value.size > 0);
  },
);

function setCandidateDirty(candidateId: string, isDirty: boolean) {
  const next = new Set(namedCandidateIds.value);
  if (isDirty) next.add(candidateId);
  else next.delete(candidateId);
  namedCandidateIds.value = next;
  emit("dirty", next.size > 0);
}
</script>

<template>
  <div class="divide-y divide-stroke-divider border-t border-stroke-divider">
    <VoiceCandidateReview
      v-for="candidate in candidates"
      :key="candidate.candidateId"
      :candidate
      :disabled
      @audition="emit('audition', candidate.candidateId)"
      @dirty="setCandidateDirty(candidate.candidateId, $event)"
      @save="emit('save', candidate.candidateId, $event)"
      @discard="emit('discard', candidate.candidateId)"
    />
  </div>
</template>
