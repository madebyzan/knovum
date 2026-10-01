import { i18n } from "../../../foundation/i18n/i18n";
import type { EnrichmentJobState } from "../contracts/generated/EnrichmentJobState";

export function enrichmentJobTitle(jobType: EnrichmentJobState["jobType"]) {
  return i18n.global.t(`spaces.processing.enrichmentJobType.${jobType}`);
}

export function runningEnrichmentTask(record: EnrichmentJobState) {
  return { id: record.id, title: enrichmentJobTitle(record.jobType), startedAt: record.createdAt };
}

export function failedEnrichmentTask(record: EnrichmentJobState) {
  if (!record.error) throw new Error("Failed enrichment task is missing its error");
  return {
    id: record.id,
    title: enrichmentJobTitle(record.jobType),
    statusMessage: record.error,
    startedAt: record.createdAt,
  };
}
