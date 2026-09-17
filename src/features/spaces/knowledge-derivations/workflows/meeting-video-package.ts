import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import { groundedDocumentSources } from "../catalog/grounded-document";
import { createGroundedDocument } from "../grounded-document";
import { executePersistedWorkflow } from "../orchestration/persisted-workflow";
import { loadWorkflow } from "../orchestration/workflow-registry";
import { createSubtitleResource } from "../subtitle-extraction";
import { createSubtitledVideo, subtitleVideoInputs } from "../subtitled-video";
import { nodeStepOutput } from "./output";

type Store = ReturnType<typeof useSpaceStore>;
type Action = "extract-subtitles" | "summarize-meeting" | "render-subtitled-video";

export interface MeetingVideoPackageDraft {
  summaryTitle: string;
  summaryPurpose: string;
  textTarget?: ExecutionTarget | undefined;
}

export async function createMeetingVideoPackage(
  store: Store,
  source: SpaceCanvasNode,
  draft: MeetingVideoPackageDraft,
) {
  const spaceId = store.activeSpaceId;
  const sources = groundedDocumentSources([source]);
  if (!spaceId || !sources) return;
  const workflow = await loadWorkflow<Action>("meeting-video-package");
  return executePersistedWorkflow<Action, SpaceCanvasNode>({
    spaceId,
    workflow,
    task: {
      title: i18n.global.t("spaces.creation.meetingVideoPackage"),
      detail: source.data.title,
    },
    adapters: {
      "extract-subtitles": async () => {
        const output = await createSubtitleResource(store, source);
        if (!output) throw new Error("Meeting subtitles were not created");
        return nodeStepOutput(output);
      },
      "summarize-meeting": async () => {
        const completed = await createGroundedDocument({
          store,
          selected: [source],
          sources,
          draft: {
            title: draft.summaryTitle,
            purpose: draft.summaryPurpose,
            target: draft.textTarget,
          },
        });
        const output = completed && store.nodes.find(({ id }) => id === completed.nodeId);
        if (!output) throw new Error("Meeting summary was not created");
        return nodeStepOutput(output);
      },
      "render-subtitled-video": async ({ inputs: stepInputs }) => {
        const subtitles = stepInputs.subtitles?.output;
        const videoInputs = subtitles && subtitleVideoInputs([source, subtitles]);
        if (!videoInputs) throw new Error("Meeting subtitle inputs are invalid");
        const output = await createSubtitledVideo(store, videoInputs);
        if (!output) throw new Error("Meeting subtitled video was not created");
        return nodeStepOutput(output);
      },
    },
  });
}
