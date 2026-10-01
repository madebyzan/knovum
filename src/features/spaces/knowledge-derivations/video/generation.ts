import type { ExecutionTarget } from "../../../../foundation/ai/schema/models";
import { i18n } from "../../../../foundation/i18n/i18n";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { startDerivationAction } from "../start-action";

export interface VideoDraft {
  title: string;
  prompt: string;
  aspectRatio: "16:9" | "9:16" | "1:1";
  durationSeconds: 5 | 10 | 15;
  target?: ExecutionTarget | undefined;
}

export function generateVideoFromSources({
  store,
  sources,
  draft,
}: {
  store: ReturnType<typeof useSpaceStore>;
  sources: readonly GroundedDocumentSource[];
  draft: VideoDraft;
}) {
  return startDerivationAction({
    store,
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
