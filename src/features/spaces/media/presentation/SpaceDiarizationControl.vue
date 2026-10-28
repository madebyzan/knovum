<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { canStartWith, type AiRequirement } from "../../../../foundation/ai/ai-readiness";
import { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { refreshEnrichmentJobs } from "../../enrichment/enrichment-jobs-query";
import { useSpaceStore } from "../../session/space-store";
import { requestMediaDiarization } from "../transcription";

const props = defineProps<{
  nodeId: string;
  revision: number;
  durationMs: number;
  speakerCount: number;
  disabled: boolean;
}>();
const { t } = useI18n();
const store = useSpaceStore();
const REQUIREMENTS: readonly AiRequirement[] = [{ kind: "localCapability", capability: "speechDiarization" }];
const readiness = useAiReadiness(REQUIREMENTS);
const numSpeakers = shallowRef("");
const submitting = shallowRef(false);
const error = shallowRef<string>();
const countValid = computed(
  () =>
    numSpeakers.value === "" ||
    (Number.isInteger(Number(numSpeakers.value)) &&
      Number(numSpeakers.value) >= 1 &&
      Number(numSpeakers.value) <= 32),
);
const canStart = computed(
  () => countValid.value && props.durationMs <= 7_200_000 && canStartWith(readiness.value),
);

async function start() {
  const spaceId = store.activeSpaceId;
  if (!spaceId || props.disabled || submitting.value || !canStart.value) return;
  submitting.value = true;
  error.value = undefined;
  try {
    if (!(await admitAiRequirements(REQUIREMENTS))) return;
    await requestMediaDiarization({
      spaceId,
      nodeId: props.nodeId,
      expectedRevision: props.revision,
      numSpeakers: numSpeakers.value ? Number(numSpeakers.value) : null,
    });
    await refreshEnrichmentJobs();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <section class="mt-3 space-y-2 rounded-panel border border-stroke-divider p-2.5">
    <p class="text-caption font-medium text-fg-secondary">
      {{ t("spaces.transcript.speakers.title") }}<span v-if="speakerCount > 0"> · {{ speakerCount }}</span>
    </p>
    <p v-if="durationMs > 7_200_000" class="text-caption text-fg-tertiary">
      {{ t("spaces.transcript.speakers.durationLimit") }}
    </p>
    <label class="block text-caption text-fg-secondary">
      {{ t("spaces.transcript.speakers.count") }}
      <UiInput
        v-model="numSpeakers"
        type="number"
        size="sm"
        min="1"
        max="32"
        :placeholder="t('spaces.transcript.speakers.automatic')"
        :disabled="disabled || submitting"
      />
    </label>
    <UiButton size="sm" :disabled="disabled || submitting || !canStart" @click="start">{{
      t("spaces.transcript.speakers.generate")
    }}</UiButton>
    <AiReadinessNotice :readiness="readiness" />
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
