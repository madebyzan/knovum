<script setup lang="ts">
import { Users } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { canStartWith, type AiRequirement } from "../../../../foundation/ai/ai-readiness";
import { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import { preferredLocalModel } from "../../../../foundation/ai/local-model-choice";
import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import { useLocalModelOptions } from "../../../../foundation/ai/use-model-choice-options";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import { refreshEnrichmentJobs } from "../../enrichment/enrichment-jobs-query";
import { useSpaceStore } from "../../session/space-store";
import { requestMediaDiarization } from "../transcription";

const props = defineProps<{
  nodeId: string;
  revision: number;
  durationMs: number;
  speakerCount: number;
  activeJob?: EnrichmentJobState | undefined;
  disabled: boolean;
}>();
const { t } = useI18n();
const store = useSpaceStore();
const REQUIREMENTS: readonly AiRequirement[] = [{ kind: "localCapability", capability: "speechDiarization" }];
const readiness = useAiReadiness(REQUIREMENTS);
const { options: diarizationModels } = useLocalModelOptions(() => "speechDiarization");
const submitting = shallowRef(false);
const error = shallowRef<string>();
const runningStatus = computed(() =>
  props.activeJob?.status === "running" || props.activeJob?.status === "pending"
    ? props.activeJob.status
    : undefined,
);
const canStart = computed(() => props.durationMs <= 7_200_000 && canStartWith(readiness.value));

async function start(chosenModel?: LocalModelId) {
  const spaceId = store.activeSpaceId;
  if (!spaceId || props.disabled || submitting.value || !canStart.value) return;
  submitting.value = true;
  error.value = undefined;
  try {
    if (!(await admitAiRequirements(REQUIREMENTS))) return;
    const selectedModel = chosenModel ?? (await preferredLocalModel("speechDiarization"));
    if (!selectedModel) return;
    await requestMediaDiarization({
      spaceId,
      nodeId: props.nodeId,
      expectedRevision: props.revision,
      numSpeakers: null,
      model: selectedModel,
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
  <section class="mb-2 space-y-1.5">
    <div class="flex min-h-interaction items-center gap-2">
      <Users class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      <span class="min-w-0 flex-1 truncate text-caption text-fg-secondary">
        {{ t("spaces.transcript.speakers.title") }}
        <span v-if="speakerCount > 0" class="text-fg-tertiary tabular-nums">· {{ speakerCount }}</span>
      </span>
      <ModelChoiceTrigger
        :options="diarizationModels"
        :disabled="disabled || submitting || !canStart || Boolean(runningStatus)"
        @select="(id) => void start(id as LocalModelId)"
      >
        <template #trigger="{ multiple }">
          <UiButton
            variant="secondary"
            size="sm"
            class="px-2.5 text-caption shadow-none"
            :disabled="disabled || submitting || !canStart || Boolean(runningStatus)"
            @click="!multiple && start()"
          >
            <UiSpinner v-if="runningStatus" size="sm" />
            <span>{{
              runningStatus
                ? t(`spaces.processing.jobStatus.${runningStatus}`)
                : t("spaces.transcript.speakers.generate")
            }}</span>
          </UiButton>
        </template>
      </ModelChoiceTrigger>
    </div>
    <p v-if="durationMs > 7_200_000" class="text-caption-sm text-fg-tertiary">
      {{ t("spaces.transcript.speakers.durationLimit") }}
    </p>
    <AiReadinessNotice :readiness="readiness" />
    <p v-if="error" class="text-caption-sm text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
