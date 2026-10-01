import type { Ref } from "vue";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { formatBytes } from "../../../foundation/format/bytes";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceMediaPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceMediaAnalysisPreview } from "./analysis-preview";
import { generateMediaInsights as generateInsights } from "./insights";
import type { SpaceMediaPlaybackState, SpaceMediaSeekRequest, SpaceMediaToggleRequest } from "./playback";
import { requestMediaTranscription } from "./transcription";

export interface SpaceMediaAnalysisLabels {
  title: string;
  preparing: string;
  transcoding: string;
  transcribing: string;
  organizing: string;
}
interface Context {
  nodes: Ref<SpaceCanvasNode[]>;
  activeSpaceId: Ref<string | undefined>;
  applyNodeContentForSpace(spaceId: string, id: string, data: SpaceNodePayload): void;
  analysisIds: Ref<string[]>;
  analysisPreviews: Ref<Record<string, SpaceMediaAnalysisPreview>>;
  seekRequest: Ref<SpaceMediaSeekRequest | undefined>;
  toggleRequest: Ref<SpaceMediaToggleRequest | undefined>;
  playback: Ref<{ id: string; state: SpaceMediaPlaybackState } | undefined>;
}

export function createSpaceMediaActions(context: Context) {
  let seekNonce = 0;
  let toggleNonce = 0;
  async function analyzeMedia(id: string) {
    const spaceId = requireSpace();
    return requestMediaTranscription(spaceId, id);
  }
  async function generateMediaInsights(id: string, labels: SpaceMediaAnalysisLabels) {
    if (context.analysisIds.value.includes(id)) return;
    const spaceId = requireSpace();
    const node = context.nodes.value.find((candidate) => candidate.id === id);
    if (!node || !isSpaceMediaPayload(node.data) || !node.data.knowledgeRevision)
      throw new Error("Current media knowledge is unavailable");
    const jobId = crypto.randomUUID();
    const tasks = useTaskStore();
    tasks.start(labels.title, labels.organizing, () => cancelAiJob(jobId), jobId);
    context.analysisIds.value = [...context.analysisIds.value, id];
    try {
      const data = await generateInsights(
        { spaceId, nodeId: id, jobId, expectedRevision: node.data.knowledgeRevision },
        ({ stage, progress, resources }) => {
          const statusMessage = resources
            ? `${labels.preparing} · ${formatBytes(resources.downloaded)}${resources.total === null ? "" : ` / ${formatBytes(resources.total)}`}`
            : stage;
          tasks.progress(jobId, progress, statusMessage);
        },
      );
      context.applyNodeContentForSpace(spaceId, id, data);
      tasks.succeed(jobId);
    } catch (error) {
      if (decodeAppError(error)?.code === "media_insights_cancelled") tasks.cancelled(jobId);
      else {
        tasks.fail(jobId, getErrorMessage(error));
        throw error;
      }
    } finally {
      context.analysisIds.value = context.analysisIds.value.filter((candidate) => candidate !== id);
    }
  }
  function requireSpace() {
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A persisted Space is required");
    return spaceId;
  }
  function requestMediaSeek(id: string, positionMs: number) {
    context.seekRequest.value = { id, positionMs, nonce: ++seekNonce };
  }
  function consumeMediaSeek(nonce: number) {
    if (context.seekRequest.value?.nonce === nonce) context.seekRequest.value = undefined;
  }
  function requestMediaToggle(id: string) {
    context.toggleRequest.value = { id, nonce: ++toggleNonce };
  }
  function consumeMediaToggle(nonce: number) {
    if (context.toggleRequest.value?.nonce === nonce) context.toggleRequest.value = undefined;
  }
  function updateMediaPlayback(id: string, state: SpaceMediaPlaybackState) {
    context.playback.value = { id, state };
  }
  function clearMediaPlayback(id: string) {
    if (context.playback.value?.id === id) context.playback.value = undefined;
  }
  return {
    analyzeMedia,
    generateMediaInsights,
    requestMediaSeek,
    consumeMediaSeek,
    requestMediaToggle,
    consumeMediaToggle,
    updateMediaPlayback,
    clearMediaPlayback,
  };
}
