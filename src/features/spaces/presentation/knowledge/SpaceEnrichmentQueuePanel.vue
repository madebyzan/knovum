<script setup lang="ts">
import { useIntervalFn } from "@vueuse/core";
import { ref } from "vue";
import { useI18n } from "vue-i18n";

import {
  cancelEnrichmentJob,
  listEnrichmentJobs,
  type SpaceEnrichmentJobRecord,
} from "../../enrichment/enrichment-jobs";

const props = defineProps<{ spaceId: string }>();

const { t } = useI18n({ useScope: "global" });
const jobs = ref<SpaceEnrichmentJobRecord[]>([]);

async function refresh() {
  jobs.value = await listEnrichmentJobs(props.spaceId);
}

async function cancel(jobId: string) {
  await cancelEnrichmentJob(jobId);
  await refresh();
}

useIntervalFn(refresh, 2_000, { immediate: true });
</script>

<template>
  <section
    v-if="jobs.length"
    class="rounded-panel border border-divider bg-surface/45 p-3 text-caption text-foreground-secondary"
  >
    <header class="mb-2 flex items-center justify-between">
      <span class="font-semibold text-foreground">{{ t("spaces.processing.enrichmentQueueTitle") }}</span>
      <span class="text-micro text-foreground-tertiary">{{
        t("spaces.processing.enrichmentQueueSubtitle", { count: jobs.length })
      }}</span>
    </header>
    <ul class="space-y-1.5">
      <li v-for="job in jobs" :key="job.id" class="flex items-center justify-between gap-3">
        <span class="min-w-0 truncate">
          {{ t(`spaces.processing.enrichmentJobType.${job.jobType}`) }}
          · <span class="text-foreground-tertiary">{{ job.status }}</span>
        </span>
        <button
          type="button"
          class="text-micro text-foreground-tertiary hover:text-foreground"
          @click="cancel(job.id)"
        >
          {{ t("spaces.processing.enrichmentQueueCancel") }}
        </button>
      </li>
    </ul>
  </section>
</template>
