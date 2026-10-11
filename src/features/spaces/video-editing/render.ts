import { Channel } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";
import { omit } from "es-toolkit";
import { parse } from "pathe";
import { toast } from "vue-sonner";
import { z } from "zod";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { CompletedDerivationOutput } from "../contracts/generated/CompletedDerivationOutput";
import type { RenderVideoEditRequest } from "../contracts/generated/RenderVideoEditRequest";
import type { VideoEditProgress } from "../contracts/generated/VideoEditProgress";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { useSpaceStore } from "../session/space-store";
import type { VideoEditProject } from "./project";

const videoEditProgressSchema: z.ZodType<VideoEditProgress> = z
  .object({ stage: z.string().min(1), progress: z.number().int().min(0).max(100) })
  .strict();

export async function renderVideoEdit(source: SpaceCanvasNode<SpaceVideoPayload>, project: VideoEditProject) {
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const spaceId = store.activeSpaceId;
  const sourceSurface = store.surface;
  const sourceRevision = source.data.sourceRevision;
  if (!spaceId || !sourceRevision) throw new Error("Video source revision is unavailable");
  const assetPath = await invokeCommand<string>("plugin:spaces|allocate_video_edit_output_path");
  const sourceContent = omit(source.data, [
    "videoEditProject",
    "sourceRevision",
    "knowledgeRevision",
    "previewPath",
    "previewText",
    "size",
    "modifiedAt",
    "mediaKnowledge",
    "mediaAnalysis",
  ]);
  const pendingContent: SpaceVideoPayload = {
    ...sourceContent,
    title: editedName(source.data.title),
    fileName: editedName(source.data.fileName),
    assetPath,
    processingState: "processing",
  };
  const [pending] = await store.createPendingContentForSpace(spaceId, pendingContent);
  if (!pending) throw new Error("Could not create the edited video output");
  store.finalizePendingContent(spaceId, [pending], false);
  const derivationId = crypto.randomUUID();
  const cancel = () => invokeCommand<void>("plugin:spaces|cancel_video_edit_render", { derivationId });
  const taskId = tasks.start(
    i18n.global.t("spaces.media.videoEditor.render.task"),
    source.data.title,
    cancel,
  );
  const progress = new Channel<VideoEditProgress>((event) => {
    const { stage, progress: value } = videoEditProgressSchema.parse(event);
    tasks.progress(taskId, value, i18n.global.t(`spaces.media.videoEditor.render.stages.${stage}`));
  });
  try {
    const completed = await invokeCommand<CompletedDerivationOutput>("plugin:spaces|render_video_edit", {
      request: {
        id: derivationId,
        spaceId,
        sourceNodeId: source.id,
        outputNodeId: pending.id,
        sourceRevision,
        project,
      } satisfies RenderVideoEditRequest,
      onProgress: progress,
    });
    const videoPayload = validateCompletedVideoPayload(completed, pending.id);
    pending.data = videoPayload;
    store.applyNodeContentForSpace(spaceId, completed.nodeId, videoPayload);
    tasks.succeed(taskId);
    await refreshOriginSpace();
    revealRenderedVideo(completed.nodeId);
  } catch (error) {
    if (decodeAppError(error)?.code === "video_edit_render_cancelled") tasks.dismiss(taskId);
    else {
      tasks.fail(taskId, getErrorMessage(error));
      toast.error(`${i18n.global.t("spaces.media.videoEditor.render.failed")}: ${getErrorMessage(error)}`);
    }
    await refreshOriginSpace();
  }

  function revealRenderedVideo(outputNodeId: string) {
    const surface = store.surface;
    if (
      store.activeSpaceId !== spaceId ||
      surface !== sourceSurface ||
      surface.kind !== "video" ||
      surface.nodeId !== source.id
    )
      return;
    store.closeVideoEditor();
    store.requestReveal([outputNodeId], "ensure-visible");
  }

  async function refreshOriginSpace() {
    if (store.activeSpaceId !== spaceId) return;
    try {
      await store.refreshActiveSpace();
    } catch (error) {
      void logError(`Video edit Space refresh failed: ${formatErrorDiagnostic(error)}`);
    }
  }
}

function validateCompletedVideoPayload(completed: CompletedDerivationOutput, pendingNodeId: string) {
  if (completed.nodeId !== pendingNodeId) throw new Error("Video render returned a different output node");
  const payload = validateSpaceNodePayload(completed.data);
  if (payload.kind !== "video") throw new Error("Video render returned non-video content");
  return payload;
}

function editedName(value: string) {
  return `${parse(value).name}-edited.mp4`;
}
