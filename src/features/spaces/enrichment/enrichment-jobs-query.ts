import { queryOptions } from "@tanstack/vue-query";

import { queryClient } from "../../../foundation/query/query-client";
import { listActionableEnrichmentJobs } from "./enrichment-jobs";

/**
 * Real-time status transitions and progress arrive over the `space-enrichment-update` push event
 * (`use-enrichment-job-observation.ts`). Keep only a low-frequency fallback reconciliation interval
 * while jobs are active so missed events self-heal without redundant IPC churn.
 */
const RECONCILE_INTERVAL_MS = 30_000;

const ENRICHMENT_JOBS_KEY = ["spaces", "enrichment-jobs"] as const;

export function enrichmentJobsQuery(enabled = true) {
  return queryOptions({
    queryKey: ENRICHMENT_JOBS_KEY,
    queryFn: listActionableEnrichmentJobs,
    enabled,
    refetchInterval: (query) =>
      query.state.data?.some(({ job }) => job.status === "pending" || job.status === "running")
        ? RECONCILE_INTERVAL_MS
        : false,
  });
}

export function refreshEnrichmentJobs() {
  return queryClient.invalidateQueries({ queryKey: ENRICHMENT_JOBS_KEY });
}
