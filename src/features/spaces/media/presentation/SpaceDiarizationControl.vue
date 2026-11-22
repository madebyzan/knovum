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
const numSpeakers = shallowRef("");
const submitting = shallowRef(false);
const error = shallowRef<string>();
const runningStatus = computed(() =>
  props.activeJob?.status === "running" || props.activeJob?.status === "pending"
    ? props.activeJob.status
    : undefined,
);
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

function onCountInput(event: Event) {
  numSpeakers.value = (event.target as HTMLInputElement).value;
}

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
      numSpeakers: numSpeakers.value ? Number(numSpeakers.value) : null,
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
  <section class="mb-2.5 space-y-1.5">
    <div class="flex items-center gap-2">
      <Users class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      <span class="min-w-0 flex-1 truncate text-caption text-fg-secondary">
        {{ t("spaces.transcript.speakers.title") }}
        <span v-if="speakerCount > 0" class="text-fg-tertiary tabular-nums">· {{ speakerCount }}</span>
      </span>
      <div
        class="inline-flex h-(--knovum-control-height-sm) shrink-0 items-center overflow-hidden rounded-md border border-stroke-default/80 bg-surface-raised shadow-control transition-colors focus-within:border-stroke-strong"
      >
        <input
          :value="numSpeakers"
          type="number"
          min="1"
          max="32"
          class="h-full w-18 [appearance:textfield] border-r border-stroke-divider/80 bg-surface-control/45 px-2 text-center text-caption-sm text-fg-primary tabular-nums outline-none placeholder:text-fg-tertiary focus:bg-surface-raised disabled:opacity-45 [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
          :aria-label="t('spaces.transcript.speakers.count')"
          :title="t('spaces.transcript.speakers.count')"
          :placeholder="t('spaces.transcript.speakers.automatic')"
          :disabled="disabled || submitting || Boolean(runningStatus)"
          @input="onCountInput"
        />
        <ModelChoiceTrigger
          :options="diarizationModels"
          :disabled="disabled || submitting || !canStart || Boolean(runningStatus)"
          @select="(id) => void start(id as LocalModelId)"
        >
          <template #trigger="{ multiple }">
            <button
              type="button"
              class="inline-flex h-full items-center gap-1.5 px-2.5 text-caption font-medium text-fg-primary transition-colors hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-stroke-focus active:bg-surface-control-pressed disabled:pointer-events-none disabled:opacity-45"
              :disabled="disabled || submitting || !canStart || Boolean(runningStatus)"
              @click="!multiple && start()"
            >
              <UiSpinner v-if="runningStatus" size="sm" />
              <span>{{
                runningStatus
                  ? t(`spaces.processing.jobStatus.${runningStatus}`)
                  : t("spaces.transcript.speakers.generate")
              }}</span>
            </button>
          </template>
        </ModelChoiceTrigger>
      </div>
    </div>
    <p v-if="durationMs > 7_200_000" class="text-caption-sm text-fg-tertiary">
      {{ t("spaces.transcript.speakers.durationLimit") }}
    </p>
    <AiReadinessNotice :readiness="readiness" />
    <p v-if="error" class="text-caption-sm text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
