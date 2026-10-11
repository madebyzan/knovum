import { i18n } from "../../../../foundation/i18n/instance";
import type { TaskTargetRef } from "../../../../shared/contracts/generated/TaskTargetRef";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { startDerivationAction } from "../start-action";

export interface VideoDraft {
  title: string;
  prompt: string;
  aspectRatio: "16:9" | "9:16" | "1:1";
  durationSeconds: 5 | 10 | 15;
  target?: TaskTargetRef | undefined;
}

interface VideoGenerationInput {
  sources: readonly GroundedDocumentSource[];
  draft: VideoDraft;
}

export function generateVideoFromSources({ sources, draft }: VideoGenerationInput) {
  return startDerivationAction({
    sources,
    actionId: "generate-video",
    title: i18n.global.t("spaces.creation.video.generation.task"),
    target: draft.target,
    parameters: {
      title: draft.title.trim(),
      prompt: draft.prompt.trim(),
      aspectRatio: draft.aspectRatio,
      durationSeconds: draft.durationSeconds,
    },
  });
}
