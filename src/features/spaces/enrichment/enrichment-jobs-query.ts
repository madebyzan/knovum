import { queryOptions } from "@tanstack/vue-query";

import { listEnrichmentJobs } from "./enrichment-jobs";

const POLL_INTERVAL_MS = 2_000;

export function enrichmentJobsKey(spaceId: string) {
  return ["spaces", spaceId, "enrichment-jobs"] as const;
}

export function enrichmentJobsQuery(spaceId: string, enabled = true) {
  return queryOptions({
    queryKey: enrichmentJobsKey(spaceId),
    queryFn: () => listEnrichmentJobs(spaceId),
    enabled,
    refetchInterval: POLL_INTERVAL_MS,
  });
}
