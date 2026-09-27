import { invokeCommand } from "../../../foundation/ipc/invoke-command";
export type { EnrichmentJobType as SpaceEnrichmentJobType } from "../contracts/generated/EnrichmentJobType";
export type { EnrichmentJobState as SpaceEnrichmentJobState } from "../contracts/generated/EnrichmentJobState";
export type { MediaTranscriptionProgress as SpaceEnrichmentJobProgress } from "../contracts/generated/MediaTranscriptionProgress";
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
