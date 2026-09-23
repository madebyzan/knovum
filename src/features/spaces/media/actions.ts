import type { Ref } from "vue";

import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceMediaPayload, type SpaceMediaPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { runSpaceMediaInference } from "../session/actions/work-scheduler";
import {
  appendSpaceMediaAnalysisPreview,
  clearSpaceMediaAnalysisPreview,
  type SpaceMediaAnalysisPreview,
} from "./analysis-preview";
import { createLocalMediaKnowledge, generateSpaceMediaInsights } from "./knowledge";
import type { SpaceMediaPlaybackState, SpaceMediaSeekRequest, SpaceMediaToggleRequest } from "./playback";
import type { SpaceMediaKnowledge } from "./schema";
import {
  cancelSpaceMediaTranscription,
  isMediaTranscriptionCancellation,
  MEDIA_TRANSCRIPTION_CANCELLED,
  transcribeSpaceMedia,
  type SpaceMediaTranscriptionStage,
} from "./transcription";

export interface SpaceMediaAnalysisLabels {
  title: string;
  preparing: string;
  transcoding: string;
  transcribing: string;
  organizing: string;
}
interface Context {
  nodes: Ref<SpaceCanvasNode[]>;
  analysisIds: Ref<string[]>;
  analysisPreviews: Ref<Record<string, SpaceMediaAnalysisPreview>>;
  seekRequest: Ref<SpaceMediaSeekRequest | undefined>;
  toggleRequest: Ref<SpaceMediaToggleRequest | undefined>;
  playback: Ref<{ id: string; state: SpaceMediaPlaybackState } | undefined>;
  saveNodeData: (id: string, expectedData: SpaceNodePayload, data: SpaceNodePayload) => Promise<void>;
}

export function createSpaceMediaActions(context: Context) {
  let seekNonce = 0;
  let toggleNonce = 0;
  async function analyzeMedia(id: string, labels: SpaceMediaAnalysisLabels) {
    const jobId = crypto.randomUUID();
    let cancelRequested = false;
    return runKnowledgeTask(
      id,
      labels,
      async (node, taskId) => {
        const tasks = useTaskStore();
        if (!isSpaceMediaPayload(node.data)) throw new Error("Media node type changed during analysis");
        const data = node.data;
        const revision = requireMediaSource(data);
        const transcript = await runSpaceMediaInference(() => {
          if (cancelRequested) throw new Error(MEDIA_TRANSCRIPTION_CANCELLED);
          return transcribeSpaceMedia(jobId, data.assetPath, ({ stage, progress, partial }) => {
            tasks.progress(taskId, progress, stageLabel(stage, labels));
            if (partial)
              appendSpaceMediaAnalysisPreview({
                previews: context.analysisPreviews,
                nodeId: id,
                progress,
                partial,
              });
          });
        });
        const knowledge = createLocalMediaKnowledge(transcript, revision);
        await saveCurrentKnowledge(id, revision, knowledge);
        return knowledge;
      },
      async () => {
        cancelRequested = true;
        await cancelSpaceMediaTranscription(jobId);
      },
    );
  }
  async function generateMediaInsights(id: string, labels: SpaceMediaAnalysisLabels) {
    return runKnowledgeTask(id, labels, async (node, taskId) => {
      if (!isSpaceMediaPayload(node.data)) throw new Error("Media node type changed during analysis");
      const data = node.data;
      const revision = requireMediaSource(data);
      const knowledge = data.mediaKnowledge;
      if (!knowledge || knowledge.sourceRevision !== revision)
        throw new Error("Current media transcript is unavailable");
      useTaskStore().progress(taskId, 30, labels.organizing);
      const next = await generateSpaceMediaInsights(knowledge);
      await saveCurrentKnowledge(id, revision, next);
      return next;
    });
  }
  async function runKnowledgeTask(
    id: string,
    labels: SpaceMediaAnalysisLabels,
    operation: (node: SpaceCanvasNode, taskId: string) => Promise<SpaceMediaKnowledge>,
    cancel?: () => Promise<void>,
  ) {
    if (context.analysisIds.value.includes(id)) return;
    const node = context.nodes.value.find((candidate) => candidate.id === id);
    if (!node) throw new Error("Media node no longer exists");
    context.analysisIds.value = [...context.analysisIds.value, id];
    clearSpaceMediaAnalysisPreview(context.analysisPreviews, id);
    const tasks = useTaskStore();
    const taskId = tasks.start(labels.title, labels.preparing, cancel);
    try {
      const knowledge = await operation(node, taskId);
      tasks.succeed(taskId);
      return knowledge;
    } catch (error) {
      if (isMediaTranscriptionCancellation(error)) {
        tasks.dismiss(taskId);
        return undefined;
      }
      tasks.fail(taskId, String(error));
      throw error;
    } finally {
      context.analysisIds.value = context.analysisIds.value.filter((candidate) => candidate !== id);
      clearSpaceMediaAnalysisPreview(context.analysisPreviews, id);
    }
  }
  async function saveCurrentKnowledge(id: string, revision: string, knowledge: SpaceMediaKnowledge) {
    const current = context.nodes.value.find((candidate) => candidate.id === id);
    if (!current || !isSpaceMediaPayload(current.data) || current.data.sourceRevision !== revision)
      throw new Error("Media source changed during analysis");
    await context.saveNodeData(id, current.data, { ...current.data, mediaKnowledge: knowledge });
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

function requireMediaSource(data: SpaceMediaPayload) {
  if (!data.sourceRevision) throw new Error("Media source revision is unavailable");
  return data.sourceRevision;
}
function stageLabel(stage: SpaceMediaTranscriptionStage, labels: SpaceMediaAnalysisLabels) {
  if (stage === "transcoding") return labels.transcoding;
  if (stage === "transcribing" || stage === "parsing") return labels.transcribing;
  return labels.preparing;
}
