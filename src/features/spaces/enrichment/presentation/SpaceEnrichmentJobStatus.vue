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
  <p
    v-if="job?.status === 'pending' || job?.status === 'running'"
    class="mt-2 text-caption leading-relaxed text-fg-tertiary"
    role="status"
  >
    {{ t(`spaces.processing.enrichmentJobType.${job.jobType}`) }} ·
    {{ t(`spaces.processing.jobStatus.${job.status}`) }}
  </p>
  <div
    v-else-if="job?.status === 'awaiting_configuration'"
    class="flex items-center justify-between gap-2 text-caption leading-relaxed text-fg-tertiary"
  >
    <span class="min-w-0">{{ t("spaces.processing.visual.waitingConfiguration") }}</span>
    <UiButton v-if="requirement" size="sm" variant="ghost" @click="configure">
      <Settings2 class="size-3" />{{ t("tasks.resolve") }}
    </UiButton>
  </div>
  <div
    v-else-if="job?.status === 'failed'"
    class="flex items-center justify-between gap-2 text-caption text-fg-danger"
  >
    <span class="min-w-0 wrap-break-word">{{ job.error ?? t("spaces.processing.failed") }}</span>
    <UiButton size="sm" variant="ghost" :disabled="retrying" @click="retry">
      <UiSpinner v-if="retrying" size="sm" />
      <RefreshCw v-else class="size-3" />
      {{ t("common.retry") }}
    </UiButton>
  </div>
</template>
