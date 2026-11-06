import { camelCase } from "es-toolkit";

import type { AiRequirement } from "../../../foundation/ai/ai-readiness";
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

/**
 * What a job runs on that the person configures. The engine parks a job whose requirement is
 * incomplete; this names where completing it happens. Local models are absent on purpose: a job
 * prepares the model it needs itself.
 */
export const ENRICHMENT_REQUIREMENTS: Partial<Record<EnrichmentJobState["jobType"], AiRequirement>> = {
  analyze_visual: { kind: "routed", purpose: "visual-understanding" },
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
