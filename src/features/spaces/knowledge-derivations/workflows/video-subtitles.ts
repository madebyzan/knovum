import { i18n } from "../../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import { executePersistedWorkflow } from "../orchestration/persisted-workflow";
import { loadWorkflow } from "../orchestration/workflow-registry";
import { createSubtitleResource } from "../subtitle-extraction";
import { createSubtitledVideo, subtitleVideoInputs } from "../subtitled-video";
import { nodeStepOutput } from "./output";

type SpaceStore = ReturnType<typeof useSpaceStore>;
type VideoSubtitleAction = "extract-subtitles" | "render-subtitled-video";

export async function createVideoWithSubtitles(store: SpaceStore, source: SpaceCanvasNode) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const tasks = useTaskStore();
  const taskId = tasks.start(i18n.global.t("spaces.creation.videoWithSubtitlesTask"), source.data.title);
  const workflow = await loadWorkflow<VideoSubtitleAction>("video-with-subtitles");
  return executePersistedWorkflow<VideoSubtitleAction, SpaceCanvasNode>({
    spaceId,
    workflow,
    task: { id: taskId, title: i18n.global.t("spaces.creation.videoWithSubtitlesTask") },
    onChange: ({ steps }) => {
      const completed = steps.filter(({ status }) => status === "succeeded").length;
      const running = steps.find(({ status }) => status === "running");
      tasks.progress(
        taskId,
        (completed / steps.length) * 100,
        running && i18n.global.t(stepLabel(running.action)),
      );
    },
    adapters: {
      "extract-subtitles": async () => {
        const subtitles = await createSubtitleResource(store, source);
        if (!subtitles) throw new Error(i18n.global.t("spaces.creation.subtitleResourceFailed"));
        return nodeStepOutput(subtitles);
      },
      "render-subtitled-video": async ({ inputs: stepInputs }) => {
        const subtitles = stepInputs.subtitles?.output;
        if (!subtitles) throw new Error("Subtitle Workflow output is unavailable");
        const inputs = subtitleVideoInputs([source, subtitles]);
        if (!inputs) throw new Error("Subtitle Workflow inputs are no longer current");
        const video = await createSubtitledVideo(store, inputs, {
          taskId,
          progressStart: 50,
          progressEnd: 100,
          completeTask: false,
        });
        if (!video) throw new Error(i18n.global.t("spaces.creation.subtitleFailed"));
        return nodeStepOutput(video);
      },
    },
  });
}

function stepLabel(action: string) {
  return action === "extract-subtitles"
    ? "spaces.creation.workflowExtractingSubtitles"
    : "spaces.creation.workflowRenderingVideo";
}
