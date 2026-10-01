import { useQuery } from "@tanstack/vue-query";
import { computed, provide, watch, type InjectionKey, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";
import { cancelEnrichmentJob, retryEnrichmentJob } from "./enrichment-jobs";
import { enrichmentJobsQuery, refreshEnrichmentJobs } from "./enrichment-jobs-query";
import { ENRICHMENT_TASK_PRODUCER, enrichmentTask } from "./job-tasks";

export const enrichmentProjectionKey: InjectionKey<Ref<EnrichmentJobProjection[]>> =
  Symbol("enrichment-jobs");

export function useEnrichmentTaskProjection(
  ready: Ref<boolean>,
  openNode: (spaceId: string, nodeId: string) => Promise<void>,
) {
  const tasks = useTaskStore();
  const query = useQuery(computed(() => enrichmentJobsQuery(ready.value)));
  const jobs = computed(() => query.data.value ?? []);
  provide(enrichmentProjectionKey, jobs);

  watch(
    query.data,
    (records) => {
      if (!records) return;
      tasks.reconcileObserved(
        ENRICHMENT_TASK_PRODUCER,
        records.map((record) => ({
          task: enrichmentTask(record),
          ...(["pending", "running"].includes(record.job.status) && {
            cancel: async () => {
              await cancelEnrichmentJob(record.job.id);
              await refreshEnrichmentJobs();
            },
          }),
          ...(record.job.status === "failed" && {
            retry: async () => {
              await retryEnrichmentJob(record.job.id);
              await refreshEnrichmentJobs();
            },
          }),
          open: () => openNode(record.job.spaceId, record.job.nodeId),
        })),
      );
    },
    { immediate: true },
  );
  watch(query.isError, (failed) => {
    if (failed && query.error.value) toast.error(getErrorMessage(query.error.value));
  });
}
