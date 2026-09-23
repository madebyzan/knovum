import { Channel, invoke } from "@tauri-apps/api/core";

import { defaultExecutionTarget } from "../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import type { SpaceMediaTranscriptionProgress } from "../media/transcription";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

export type SpaceEnrichmentJobType =
  | "analyze_image"
  | "analyze_video_cover"
  | "analyze_media"
  | "transcribe_media"
  | "parse_document"
  | "analyze_document";

export type SpaceEnrichmentJobProgress = Omit<SpaceMediaTranscriptionProgress, "stage"> & {
  stage: SpaceMediaTranscriptionProgress["stage"] | "analyzing" | "document_visual";
};

export interface SpaceEnrichmentJob {
  id: string;
  spaceId: string;
  nodeId: string;
  sourceRevision: string;
  type: SpaceEnrichmentJobType;
}

type SpaceEnrichmentJobStatus =
  | "pending"
  | "running"
  | "succeeded"
  | "failed"
  | "cancelled"
  | "awaiting_configuration";

export interface SpaceEnrichmentJobRecord {
  id: string;
  spaceId: string;
  nodeId: string;
  jobType: SpaceEnrichmentJobType;
  status: SpaceEnrichmentJobStatus;
  sourceRevision: string;
  attempts: number;
  error: string | null;
  createdAt: number;
  updatedAt: number;
}

export async function runEnrichmentJob(
  job: SpaceEnrichmentJob,
  onProgressUpdate?: (progress: SpaceEnrichmentJobProgress) => void,
) {
  let target: ExecutionTarget | null = null;
  const visual = ["analyze_image", "analyze_video_cover", "analyze_document"].includes(job.type);
  if (visual) {
    const settings = useAiSettingsStore();
    await settings.ensureLoaded();
    target = defaultExecutionTarget(settings.settings, "visual-understanding");
    if (!target) {
      await deferForConfiguration(job.id, "Visual understanding is not configured");
      return null;
    }
  }
  const onProgress = new Channel<SpaceEnrichmentJobProgress>();
  // Tauri Channel exposes a single IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  onProgress.onmessage = (progress) => onProgressUpdate?.(progress);
  return invoke<{ nodeId: string; data: SpaceNodePayload }>("plugin:spaces|run_enrichment_job", {
    request: {
      jobId: job.id,
      spaceId: job.spaceId,
      nodeId: job.nodeId,
      jobType: job.type,
      sourceRevision: job.sourceRevision,
      visionTarget: target?.type === "model" ? { ...target, temperature: 0.2 } : target,
    },
    onProgress,
  });
}

function deferForConfiguration(jobId: string, error: string) {
  return invoke<void>("plugin:spaces|defer_enrichment_for_configuration", { jobId, error });
}

export function cancelEnrichmentJob(jobId: string) {
  return invoke<void>("plugin:spaces|cancel_enrichment_job", { jobId });
}

export function listEnrichmentJobs(spaceId: string) {
  return invoke<SpaceEnrichmentJobRecord[]>("plugin:spaces|list_enrichment_jobs", { spaceId });
}
