<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import { retryEnrichmentJob } from "../../enrichment/enrichment-jobs";

const props = defineProps<{ job: EnrichmentJobState | undefined }>();
const emit = defineEmits<{ retried: [] }>();
const { t } = useI18n();
const retrying = shallowRef(false);

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
  <p
    v-else-if="job?.status === 'awaiting_configuration'"
    class="text-caption leading-relaxed text-fg-tertiary"
  >
    {{ t("spaces.processing.visual.waitingConfiguration") }}
  </p>
  <div
    v-else-if="job?.status === 'failed'"
    class="flex items-center justify-between gap-2 text-caption text-fg-danger"
  >
    <span class="min-w-0 wrap-break-word">{{ job.error ?? t("spaces.processing.failed") }}</span>
    <UiButton size="sm" variant="ghost" :disabled="retrying" @click="retry">
      <RefreshCw class="size-3" :class="retrying && 'animate-spin'" />{{ t("common.retry") }}
    </UiButton>
  </div>
</template>
