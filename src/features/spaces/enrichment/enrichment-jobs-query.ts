import { queryOptions } from "@tanstack/vue-query";

import { queryClient } from "../../../foundation/query/query-client";
import { listActionableEnrichmentJobs } from "./enrichment-jobs";

const POLL_INTERVAL_MS = 2_000;

const ENRICHMENT_JOBS_KEY = ["spaces", "enrichment-jobs"] as const;

export function enrichmentJobsQuery(enabled = true) {
  return queryOptions({
    queryKey: ENRICHMENT_JOBS_KEY,
    queryFn: listActionableEnrichmentJobs,
    enabled,
    refetchInterval: (query) =>
      query.state.data?.some(({ job }) => job.status === "pending" || job.status === "running")
        ? POLL_INTERVAL_MS
        : false,
  });
}

export function refreshEnrichmentJobs() {
  return queryClient.invalidateQueries({ queryKey: ENRICHMENT_JOBS_KEY });
}
