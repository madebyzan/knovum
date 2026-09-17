import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { useSpaceStore } from "../session/space-store";
import { listEnrichmentJobs, runEnrichmentJob, type SpaceEnrichmentJobRecord } from "./enrichment-jobs";
import { resumePendingEnrichmentJobs } from "./job-scheduler";

export interface NodeEnrichmentStatus {
  pending: number;
  running: number;
  failed: number;
  waiting: number;
  lastError?: string;
}

const POLL_INTERVAL_MS = 2_000;

export function useNodeEnrichmentStatus(spaceId: Ref<string | undefined>, nodeId: Ref<string | undefined>) {
  const store = useSpaceStore();
  const retrying = shallowRef(false);
  const query = useQuery(
    computed(() => ({
      queryKey: ["spaces", spaceId.value ?? "", "enrichment-jobs"] as const,
      queryFn: ({ queryKey }: { queryKey: readonly [string, string, string] }) =>
        listEnrichmentJobs(queryKey[1]),
      enabled: Boolean(spaceId.value && nodeId.value),
      refetchInterval: POLL_INTERVAL_MS,
    })),
  );
  const jobs = computed<SpaceEnrichmentJobRecord[]>(() => {
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
    const currentSpaceId = spaceId.value;
    if (!failed.length || !currentSpaceId || retrying.value) return;
    retrying.value = true;
    try {
      for (const job of failed) {
        const output = await runEnrichmentJob({
          id: job.id,
          spaceId: currentSpaceId,
          nodeId: job.nodeId,
          type: job.jobType,
          sourceRevision: job.sourceRevision,
        });
        if (output) store.applyNodeDataForSpace(currentSpaceId, output.nodeId, output.data);
      }
      await resumePendingEnrichmentJobs(
        { applyNodeDataForSpace: store.applyNodeDataForSpace },
        currentSpaceId,
      );
    } catch (error) {
      toast.error(String(error));
    } finally {
      retrying.value = false;
      await refresh();
    }
  }

  return { status, jobs, retrying, refresh, retryFailed };
}
