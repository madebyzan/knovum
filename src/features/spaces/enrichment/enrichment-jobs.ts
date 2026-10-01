import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";

export function cancelEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|cancel_enrichment_job", { jobId });
}
export function retryEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|retry_enrichment_job", { jobId });
}
export function listActionableEnrichmentJobs() {
  return invokeCommand<EnrichmentJobProjection[]>("plugin:spaces|list_actionable_enrichment_jobs");
}
