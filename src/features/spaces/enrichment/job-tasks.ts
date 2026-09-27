import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceEnrichmentJobState } from "./enrichment-jobs";

export function enrichmentJobTitle(jobType: SpaceEnrichmentJobState["jobType"]) {
  return i18n.global.t(`spaces.processing.enrichmentJobType.${jobType}`);
}

export function runningEnrichmentTask(record: SpaceEnrichmentJobState) {
  return { id: record.id, title: enrichmentJobTitle(record.jobType), startedAt: record.createdAt };
}

export function failedEnrichmentTask(record: SpaceEnrichmentJobState) {
  if (!record.error) throw new Error("Failed enrichment task is missing its error");
  return {
    id: record.id,
    title: enrichmentJobTitle(record.jobType),
    statusMessage: record.error,
    startedAt: record.createdAt,
  };
}
