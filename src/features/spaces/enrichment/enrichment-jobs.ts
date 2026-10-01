import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { EnrichmentJobState } from "../contracts/generated/EnrichmentJobState";

export function cancelEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|cancel_enrichment_job", { jobId });
}
export function retryEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|retry_enrichment_job", { jobId });
}
export function listEnrichmentJobs(spaceId: string) {
  return invokeCommand<EnrichmentJobState[]>("plugin:spaces|list_enrichment_jobs", { spaceId });
}
