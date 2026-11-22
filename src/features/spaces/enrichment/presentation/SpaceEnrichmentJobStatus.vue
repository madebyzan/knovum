<script setup lang="ts">
import { RefreshCw, Settings2 } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { openAiRequirementSettings } from "../../../../foundation/ai/ai-readiness-admission";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import { retryEnrichmentJob } from "../enrichment-jobs";
import { ENRICHMENT_REQUIREMENTS } from "../job-tasks";

const props = defineProps<{ job: EnrichmentJobState | undefined }>();
const emit = defineEmits<{ retried: [] }>();
const { t } = useI18n();
const retrying = shallowRef(false);
const requirement = computed(() => props.job && ENRICHMENT_REQUIREMENTS[props.job.jobType]);
const waitingMessage = computed(() => {
  if (!props.job) return "";
  if (props.job.jobType === "analyze_visual") return t("spaces.processing.visual.waitingConfiguration");
  return `${t(`spaces.processing.enrichmentJobType.${props.job.jobType}`)} · ${t("spaces.processing.jobStatus.awaiting_configuration")}`;
});

async function configure() {
  if (!requirement.value) return;
  try {
    await openAiRequirementSettings(requirement.value);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}

async function retry() {
  if (!props.job || retrying.value) return;
  retrying.value = true;
  try {
    await retryEnrichmentJob(props.job.id);
    emit("retried");
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    retrying.value = false;
  }
}
</script>

<template>
  <div
    v-if="job?.status === 'pending' || job?.status === 'running'"
    class="mt-2 flex items-center gap-2 rounded-md bg-surface-control/45 px-2.5 py-1.5 text-caption text-fg-secondary"
    role="status"
  >
    <UiSpinner size="sm" />
    <span class="min-w-0 truncate">
      {{ t(`spaces.processing.enrichmentJobType.${job.jobType}`) }} ·
      {{ t(`spaces.processing.jobStatus.${job.status}`) }}
    </span>
  </div>
  <div
    v-else-if="job?.status === 'awaiting_configuration'"
    class="mt-2 flex items-center justify-between gap-2 rounded-md bg-surface-control/45 px-2.5 py-2 text-caption leading-relaxed text-fg-secondary"
  >
    <span class="min-w-0">{{ waitingMessage }}</span>
    <UiButton v-if="requirement" size="sm" variant="secondary" @click="configure">
      <Settings2 class="size-3" />{{ t("tasks.resolve") }}
    </UiButton>
  </div>
  <div
    v-else-if="job?.status === 'failed'"
    class="mt-2 flex items-start justify-between gap-2 rounded-md border border-stroke-danger/25 bg-surface-danger-subtle px-2.5 py-2 text-caption text-fg-danger"
  >
    <span class="min-w-0 leading-relaxed wrap-break-word">
      {{ t(`spaces.processing.enrichmentJobType.${job.jobType}`) }} ·
      {{ job.error ?? t("spaces.processing.failed") }}
    </span>
    <UiButton size="sm" variant="ghost" class="shrink-0" :disabled="retrying" @click="retry">
      <UiSpinner v-if="retrying" size="sm" />
      <RefreshCw v-else class="size-3" />
      {{ t("common.retry") }}
    </UiButton>
  </div>
</template>
