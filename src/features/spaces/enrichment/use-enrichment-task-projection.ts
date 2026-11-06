import { useQuery } from "@tanstack/vue-query";
import { computed, watch, type Ref } from "vue";
import { toast } from "vue-sonner";

import { openAiRequirementSettings } from "../../../foundation/ai/ai-readiness-admission";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import type { ObservedTask } from "../../../foundation/tasks/task-observation";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";
import type { EnrichmentJobState } from "../contracts/generated/EnrichmentJobState";
import { cancelEnrichmentJob, retryEnrichmentJob } from "./enrichment-jobs";
import { enrichmentJobsQuery, refreshEnrichmentJobs } from "./enrichment-jobs-query";
import { ENRICHMENT_REQUIREMENTS, ENRICHMENT_TASK_PRODUCER, enrichmentTask } from "./job-tasks";

interface EnrichmentTaskProjectionOptions {
  ready: Ref<boolean>;
  openNode: (spaceId: string, nodeId: string) => Promise<void>;
}

export function useEnrichmentTaskProjection({ ready, openNode }: EnrichmentTaskProjectionOptions) {
  const tasks = useTaskStore();
  const query = useQuery(computed(() => enrichmentJobsQuery(ready.value)));
  const jobs = computed(() => query.data.value ?? []);

  watch(
    [query.data, i18n.global.locale],
    ([records]) => {
      if (!records) return;
      tasks.reconcileObserved(
        ENRICHMENT_TASK_PRODUCER,
        records.map((record) => observedEnrichment(record, openNode)),
      );
    },
    { immediate: true },
  );
  watch(query.isError, (failed) => {
    if (failed && query.error.value) toast.error(getErrorMessage(query.error.value));
  });

  return jobs;
}

/** A job that has not settled can still be stopped, including one waiting for configuration. */
const UNSETTLED: ReadonlySet<EnrichmentJobState["status"]> = new Set([
  "pending",
  "running",
  "awaiting_configuration",
]);

function observedEnrichment(
  record: EnrichmentJobProjection,
  openNode: EnrichmentTaskProjectionOptions["openNode"],
): ObservedTask {
  const { job } = record;
  const requirement = ENRICHMENT_REQUIREMENTS[job.jobType];
  return {
    task: enrichmentTask(record),
    ...(UNSETTLED.has(job.status) && {
      cancel: async () => {
        await cancelEnrichmentJob(job.id);
        await refreshEnrichmentJobs();
      },
    }),
    ...(job.status === "failed" && {
      retry: async () => {
        await retryEnrichmentJob(job.id);
        await refreshEnrichmentJobs();
      },
    }),
    ...(job.status === "awaiting_configuration" &&
      requirement && { resolve: () => openAiRequirementSettings(requirement) }),
    open: () => openNode(job.spaceId, job.nodeId),
  };
}
