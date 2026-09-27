import { Channel } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { decodeCommandError } from "../../../foundation/ipc/command-error";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceSubtitlePayload, SpaceVideoPayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";
import { baseNameFromNode } from "./node-naming";
import { beginPendingDerivation, endPendingDerivation } from "./pending-session";
import { derivedNodePosition } from "./placement";

export const VIDEO_SUBTITLE_DERIVATION_CANCELLED = "video_subtitle_derivation_cancelled";

export function isVideoSubtitleCancelled(error: unknown): boolean {
  return decodeCommandError(error)?.code === VIDEO_SUBTITLE_DERIVATION_CANCELLED;
}

type SpaceStore = ReturnType<typeof useSpaceStore>;
interface Progress {
  stage: string;
  progress: number;
}
interface CompletedVideo {
  nodeId: string;
  data: SpaceVideoPayload;
}

interface SubtitledVideoExecution {
  taskId?: string;
  progressStart?: number;
  progressEnd?: number;
  completeTask?: boolean;
}

export interface SubtitleVideoInputs {
  video: SpaceCanvasNode & { data: SpaceVideoPayload & { sourceRevision: string } };
  subtitles: SpaceCanvasNode & {
    data: SpaceSubtitlePayload & { knowledgeRevision: string };
  };
}

export function subtitleVideoInputs(nodes: readonly SpaceCanvasNode[]): SubtitleVideoInputs | undefined {
  if (nodes.length !== 2) return;
  const video = nodes.find((node) => node.data.kind === "video");
  const subtitles = nodes.find((node) => node.data.kind === "subtitle");
  if (!isCurrentPair(video, subtitles)) return;
  return { video, subtitles } as SubtitleVideoInputs;
}

function isCurrentPair(video?: SpaceCanvasNode, subtitles?: SpaceCanvasNode): boolean {
  if (video?.data.kind !== "video" || subtitles?.data.kind !== "subtitle") return false;
  if (!video.data.sourceRevision || !subtitles.data.knowledgeRevision) return false;
  return (
    subtitles.data.sourceVideoNodeId === video.id &&
    subtitles.data.sourceVideoRevision === video.data.sourceRevision
  );
}

export async function createSubtitledVideo(
  store: SpaceStore,
  inputs: SubtitleVideoInputs,
  execution: SubtitledVideoExecution = {},
) {
  const { video: source, subtitles } = inputs;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const pending = await createPendingNode(store, source, subtitles);
  if (!pending) return;
  beginPendingDerivation(pending.id, [source.id, subtitles.id]);
  const jobId = crypto.randomUUID();
  const tasks = useTaskStore();
  const cancel = async () => {
    await invoke("plugin:spaces|cancel_subtitled_video", { jobId });
  };
  const taskId =
    execution.taskId ??
    tasks.start(i18n.global.t("spaces.creation.video.subtitles.task"), source.data.title, cancel);
  if (execution.taskId) tasks.setCancellation(taskId, cancel);
  const channel = new Channel<Progress>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes progress through onmessage.
  channel.onmessage = ({ stage, progress }) => {
    const start = execution.progressStart ?? 0;
    const end = execution.progressEnd ?? 100;
    tasks.progress(
      taskId,
      start + (progress / 100) * (end - start),
      i18n.global.t(`spaces.creation.stages.${stage}`),
    );
  };
  try {
    await yieldToCanvas();
    const completed = await invoke<CompletedVideo>("plugin:spaces|create_subtitled_video", {
      request: {
        id: jobId,
        spaceId,
        sourceNodeId: source.id,
        outputNodeId: pending.id,
        sourceRevision: source.data.sourceRevision,
        subtitleNodeId: subtitles.id,
        subtitleRevision: subtitles.data.knowledgeRevision,
      },
      onProgress: channel,
    });
    store.applyNodeContentForSpace(spaceId, completed.nodeId, completed.data);
    await store.refreshActiveSpace();
    if (execution.completeTask !== false) tasks.succeed(taskId);
    else tasks.setCancellation(taskId);
    store.requestReveal([completed.nodeId], "ensure-visible");
    return store.nodes.find((node) => node.id === completed.nodeId);
  } catch (error) {
    await store.refreshActiveSpace();
    await markPendingVideoFailed(store, spaceId, pending.id);
    if (isVideoSubtitleCancelled(error)) tasks.dismiss(taskId);
    else {
      tasks.fail(taskId, String(error));
      toast.error(`${i18n.global.t("spaces.creation.video.subtitles.failed")}: ${String(error)}`);
    }
    return undefined;
  } finally {
    endPendingDerivation(pending.id);
  }
}

async function markPendingVideoFailed(store: SpaceStore, spaceId: string, nodeId: string) {
  const current = store.nodes.find((node) => node.id === nodeId);
  if (current?.data.kind !== "video" || current.data.processingState !== "processing") return;
  await store.saveNodeContentForSpace(spaceId, nodeId, current.data, {
    ...current.data,
    processingState: "failed",
  });
  await store.refreshActiveSpace();
}

async function createPendingNode(store: SpaceStore, source: SpaceCanvasNode, subtitles: SpaceCanvasNode) {
  if (source.data.kind !== "video") return;
  const base = baseNameFromNode(source);
  const assetPath = await invoke<string>("plugin:spaces|allocate_video_output_path");
  const data: SpaceVideoPayload = {
    kind: "video",
    title: `${base}-subtitled.mp4`,
    fileName: `${base}-subtitled.mp4`,
    assetPath,
    processingState: "processing",
  };
  const [node] = await store.addContent(
    data,
    derivedNodePosition([source, subtitles], useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  return node;
}
