import { Channel } from "@tauri-apps/api/core";
import { omit } from "es-toolkit";
import { parse } from "pathe";
import { toast } from "vue-sonner";
import { z } from "zod";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { errorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { CompletedDerivationOutput } from "../contracts/generated/CompletedDerivationOutput";
import type { RenderVideoEditRequest } from "../contracts/generated/RenderVideoEditRequest";
import type { VideoEditProgress } from "../contracts/generated/VideoEditProgress";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import type { useSpaceStore } from "../session/space-store";
import type { VideoEditProject } from "./project";

const videoEditProgressSchema: z.ZodType<VideoEditProgress> = z
  .object({ stage: z.string().min(1), progress: z.number().int().min(0).max(100) })
  .strict();

type VideoEditStore = Pick<
  ReturnType<typeof useSpaceStore>,
  | "activeSpaceId"
  | "addContent"
  | "applyNodeContentForSpace"
  | "refreshActiveSpace"
  | "closeVideoEditor"
  | "requestReveal"
>;

export async function renderVideoEdit(
  store: VideoEditStore,
  source: SpaceCanvasNode & { data: SpaceVideoPayload },
  project: VideoEditProject,
) {
  const spaceId = store.activeSpaceId;
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
  const [pending] = await store.addContent(pendingContent, undefined, { reveal: false });
  if (!pending) throw new Error("Could not create the edited video output");
  const derivationId = crypto.randomUUID();
  const tasks = useTaskStore();
  const cancel = () => invokeCommand<void>("plugin:spaces|cancel_video_edit_render", { derivationId });
  const taskId = tasks.start(
    i18n.global.t("spaces.media.videoEditor.render.task"),
    source.data.title,
    cancel,
  );
  const progress = new Channel<VideoEditProgress>((event) => {
    const { stage, progress: value } = videoEditProgressSchema.parse(event);
    tasks.progress(taskId, value, i18n.global.t(`spaces.media.videoEditor.${stage}`));
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
    if (completed.nodeId !== pending.id) throw new Error("Video render returned a different output node");
    const data = validateSpaceNodePayload(completed.data);
    if (data.kind !== "video") throw new Error("Video render returned non-video content");
    store.applyNodeContentForSpace(spaceId, completed.nodeId, data);
    await store.refreshActiveSpace();
    tasks.succeed(taskId);
    store.closeVideoEditor();
    store.requestReveal([completed.nodeId], "ensure-visible");
  } catch (error) {
    try {
      await store.refreshActiveSpace();
    } finally {
      if (decodeAppError(error)?.code === "video_edit_render_cancelled") tasks.dismiss(taskId);
      else {
        tasks.fail(taskId, errorMessage(error));
        toast.error(`${i18n.global.t("spaces.media.videoEditor.render.failed")}: ${errorMessage(error)}`);
      }
    }
  }
}

function editedName(value: string) {
  return `${parse(value).name}-edited.mp4`;
}
