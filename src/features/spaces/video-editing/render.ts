import { Channel, invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
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
  await store.updateManyData([
    {
      id: source.id,
      before: source.data,
      after: { ...source.data, videoEditProject: project },
    },
  ]);
  const { videoEditProject: _project, ...sourceData } = source.data;
  const pendingData: SpaceVideoPayload = {
    ...sourceData,
    title: editedName(source.data.title),
    fileName: editedName(source.data.fileName),
    processingState: "processing",
  };
  const [pending] = await store.addContent(pendingData, undefined, { reveal: false });
  if (!pending) throw new Error("Could not create the edited video output");
  const jobId = crypto.randomUUID();
  const tasks = useTaskStore();
  const cancel = () => invoke<void>("plugin:spaces|cancel_video_edit_render", { jobId });
  const taskId = tasks.start(i18n.global.t("spaces.media.videoRenderTask"), source.data.title, cancel);
  const progress = new Channel<Progress>(({ stage, progress: value }) => {
    tasks.progress(taskId, value, i18n.global.t(`spaces.media.${stage}`));
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
    store.applyNodeDataForSpace(spaceId, completed.nodeId, completed.data);
    await store.refreshActiveSpace();
    tasks.succeed(taskId);
    store.closeVideoEditor();
    store.requestReveal([completed.nodeId], "ensure-visible");
  } catch (error) {
    await store.refreshActiveSpace();
    if (String(error).includes("VIDEO_EDIT_RENDER_CANCELLED")) tasks.dismiss(taskId);
    else {
      tasks.fail(taskId, String(error));
      toast.error(`${i18n.global.t("spaces.media.videoRenderFailed")}: ${String(error)}`);
    }
  }
}

function editedName(value: string) {
  const separator = value.lastIndexOf(".");
  return separator > 0 ? `${value.slice(0, separator)}-edited.mp4` : `${value}-edited.mp4`;
}
