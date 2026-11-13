import { aiExecutionTargetPayload, loadAiExecutionTarget } from "../../../foundation/ai/ai-execution";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { EnrichmentJobProjection } from "../contracts/generated/EnrichmentJobProjection";

export function cancelEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|cancel_enrichment_job", { jobId });
}
export function retryEnrichmentJob(jobId: string) {
  return invokeCommand<void>("plugin:spaces|retry_enrichment_job", { jobId });
}
export async function requestVisualAnalysis(spaceId: string, nodeId: string, target: TaskTarget) {
  const executionTarget = await loadAiExecutionTarget("visual-understanding", target);
  return invokeCommand<string>("plugin:spaces|request_visual_analysis", {
    spaceId,
    nodeId,
    target: aiExecutionTargetPayload(executionTarget),
  });
}
export function listActionableEnrichmentJobs() {
  return invokeCommand<EnrichmentJobProjection[]>("plugin:spaces|list_actionable_enrichment_jobs");
}
