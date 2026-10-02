import { useQuery } from "@tanstack/vue-query";
import { computed, watch, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { cancelEnrichmentJob, retryEnrichmentJob } from "./enrichment-jobs";
import { enrichmentJobsQuery, refreshEnrichmentJobs } from "./enrichment-jobs-query";
import { ENRICHMENT_TASK_PRODUCER, enrichmentTask } from "./job-tasks";

interface EnrichmentTaskProjectionOptions {
  ready: Ref<boolean>;
  openNode: (spaceId: string, nodeId: string) => Promise<void>;
}

export function useEnrichmentTaskProjection({ ready, openNode }: EnrichmentTaskProjectionOptions) {
  const tasks = useTaskStore();
  const query = useQuery(computed(() => enrichmentJobsQuery(ready.value)));
  const jobs = computed(() => query.data.value ?? []);

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

  return jobs;
}
