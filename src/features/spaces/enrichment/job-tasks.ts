import { camelCase } from "es-toolkit";

import { i18n } from "../../../foundation/i18n/instance";
import type { AppTask, TaskStatus } from "../../../foundation/tasks/task-model";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";
import type { EnrichmentJobState } from "../contracts/generated/EnrichmentJobState";

export const ENRICHMENT_TASK_PRODUCER = "spaces-enrichment";

const TASK_STATUS_BY_JOB_STATUS: Record<EnrichmentJobState["status"], TaskStatus> = {
  pending: "queued",
  running: "running",
  awaiting_configuration: "attention",
  succeeded: "succeeded",
  failed: "failed",
  cancelled: "cancelled",
};

export function enrichmentJobTitle(jobType: EnrichmentJobState["jobType"]) {
  return i18n.global.t(`spaces.processing.enrichmentJobType.${jobType}`);
}

export function enrichmentTask(record: EnrichmentJobProjection): AppTask {
  const { job } = record;
  return {
    id: job.id,
    title: enrichmentJobTitle(job.jobType),
    status: TASK_STATUS_BY_JOB_STATUS[job.status],
    ...(job.error && { statusMessage: job.error }),
    startedAt: job.createdAt,
    scope: { kind: "space", id: job.spaceId, label: record.spaceTitle },
    subject: {
      kind: "node",
      id: job.nodeId,
      label: record.nodeTitle,
      typeLabel: i18n.global.t(`spaces.nodeTypes.${camelCase(record.nodeKind)}`),
    },
  };
}
