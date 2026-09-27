import { Channel } from "@tauri-apps/api/core";
import { parse } from "pathe";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { decodeCommandError } from "../../../foundation/ipc/command-error";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import type { VideoEditProject } from "./project";

interface Progress {
  stage: string;
  progress: number;
}
interface CompletedVideo {
  nodeId: string;
  data: SpaceVideoPayload;
}

export async function renderVideoEdit(
  store: ReturnType<typeof useSpaceStore>,
  source: SpaceCanvasNode & { data: SpaceVideoPayload },
  project: VideoEditProject,
) {
  const spaceId = store.activeSpaceId;
  const sourceRevision = source.data.sourceRevision;
  if (!spaceId || !sourceRevision) throw new Error("Video source revision is unavailable");
  const { videoEditProject: _project, ...sourceContent } = source.data;
  const pendingContent: SpaceVideoPayload = {
    ...sourceContent,
    title: editedName(source.data.title),
    fileName: editedName(source.data.fileName),
    processingState: "processing",
  };
  const [pending] = await store.addContent(pendingContent, undefined, { reveal: false });
  if (!pending) throw new Error("Could not create the edited video output");
  const jobId = crypto.randomUUID();
  const tasks = useTaskStore();
  const cancel = () => invoke<void>("plugin:spaces|cancel_video_edit_render", { jobId });
  const taskId = tasks.start(
    i18n.global.t("spaces.media.videoEditor.render.task"),
    source.data.title,
    cancel,
  );
  const progress = new Channel<Progress>(({ stage, progress: value }) => {
    tasks.progress(taskId, value, i18n.global.t(`spaces.media.videoEditor.${stage}`));
  });
  try {
    const completed = await invoke<CompletedVideo>("plugin:spaces|render_video_edit", {
      request: {
        id: jobId,
        spaceId,
        sourceNodeId: source.id,
        outputNodeId: pending.id,
        sourceRevision,
        project,
      },
      onProgress: progress,
    });
    store.applyNodeContentForSpace(spaceId, completed.nodeId, completed.data);
    await store.refreshActiveSpace();
    tasks.succeed(taskId);
    store.closeVideoEditor();
    store.requestReveal([completed.nodeId], "ensure-visible");
  } catch (error) {
    await store.refreshActiveSpace();
    if (decodeCommandError(error)?.code === "video_edit_render_cancelled") tasks.dismiss(taskId);
    else {
      tasks.fail(taskId, String(error));
      toast.error(`${i18n.global.t("spaces.media.videoEditor.render.failed")}: ${String(error)}`);
    }
  }
}

function editedName(value: string) {
  return `${parse(value).name}-edited.mp4`;
}
