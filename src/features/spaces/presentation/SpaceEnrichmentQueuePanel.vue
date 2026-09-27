<script setup lang="ts">
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { cancelEnrichmentJob } from "../enrichment/enrichment-jobs";
import { enrichmentJobsKey, enrichmentJobsQuery } from "../enrichment/enrichment-jobs-query";

const props = defineProps<{ spaceId: string }>();

const { t } = useI18n({ useScope: "global" });
const queryClient = useQueryClient();
const query = useQuery(computed(() => enrichmentJobsQuery(props.spaceId)));
const jobs = computed(() => query.data.value ?? []);
const cancellation = useMutation({
  mutationFn: cancelEnrichmentJob,
  onSuccess: () => queryClient.invalidateQueries({ queryKey: enrichmentJobsKey(props.spaceId) }),
  onError: (error) => toast.error(String(error)),
});
</script>

<template>
  <section
    v-if="jobs.length"
    class="rounded-panel border border-divider bg-surface/45 p-3 text-caption text-foreground-secondary"
  >
    <header class="mb-2 flex items-center justify-between">
      <span class="font-semibold text-foreground">{{ t("spaces.processing.queue.title") }}</span>
      <span class="text-micro text-foreground-tertiary">{{
        t("spaces.processing.queue.subtitle", { count: jobs.length })
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
          class="text-micro text-foreground-tertiary hover:text-foreground disabled:opacity-45"
          :disabled="cancellation.isPending.value"
          @click="cancellation.mutate(job.id)"
        >
          {{ t("common.cancel") }}
        </button>
      </li>
    </ul>
  </section>
</template>
