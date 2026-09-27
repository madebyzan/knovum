import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { retryEnrichmentJob, type SpaceEnrichmentJobState } from "./enrichment-jobs";
import { enrichmentJobsQuery } from "./enrichment-jobs-query";

export interface NodeEnrichmentStatus {
  pending: number;
  running: number;
  failed: number;
  waiting: number;
  lastError?: string;
}

export function useNodeEnrichmentStatus(spaceId: Ref<string | undefined>, nodeId: Ref<string | undefined>) {
  const retrying = shallowRef(false);
  const query = useQuery(
    computed(() => enrichmentJobsQuery(spaceId.value ?? "", Boolean(spaceId.value && nodeId.value))),
  );
  const jobs = computed<SpaceEnrichmentJobState[]>(() => {
    if (!spaceId.value || !nodeId.value) return [];
    return (query.data.value ?? []).filter(
      (job) => job.spaceId === spaceId.value && job.nodeId === nodeId.value,
    );
  });
  const status = computed<NodeEnrichmentStatus>(() => {
    let pending = 0;
    let running = 0;
    let failed = 0;
    let waiting = 0;
    let lastError: string | undefined;
    for (const job of jobs.value) {
      if (job.status === "pending") pending += 1;
      else if (job.status === "running") running += 1;
      else if (job.status === "awaiting_configuration") waiting += 1;
      else if (job.status === "failed") {
        failed += 1;
        lastError = job.error ?? lastError;
      }
    }
    return lastError === undefined
      ? { pending, running, failed, waiting }
      : { pending, running, failed, waiting, lastError };
  });
  async function refresh() {
    if (spaceId.value && nodeId.value) await query.refetch();
  }

  async function retryFailed() {
    const failed = jobs.value.filter((job) => job.status === "failed");
    if (!failed.length || !spaceId.value || retrying.value) return;
    retrying.value = true;
    try {
      for (const job of failed) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- Retries share the node's projection stream.
        await retryEnrichmentJob(job.id);
      }
    } catch (error) {
      toast.error(String(error));
    } finally {
      retrying.value = false;
      await refresh();
    }
  }

  return { status, jobs, retrying, refresh, retryFailed };
}
