import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceEnrichmentJobRecord } from "./enrichment-jobs";

export function enrichmentJobTitle(jobType: SpaceEnrichmentJobRecord["jobType"]) {
  return i18n.global.t(`spaces.processing.enrichmentJobType.${jobType}`);
}

export function runningEnrichmentTask(record: SpaceEnrichmentJobRecord) {
  return { id: record.id, title: enrichmentJobTitle(record.jobType), startedAt: record.createdAt };
}

export function failedEnrichmentTask(record: SpaceEnrichmentJobRecord) {
  return {
    id: record.id,
    title: enrichmentJobTitle(record.jobType),
    detail: record.error ?? i18n.global.t("spaces.processing.processingFailed"),
    startedAt: record.createdAt,
  };
}
