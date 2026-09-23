import type { useSpaceStore } from "../../session/space-store";
import { runWorkflowTemplate } from "../../workflows/template-client";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import type { VideoDraft } from "../video/generation";

type Store = ReturnType<typeof useSpaceStore>;

export async function generateVideoFromSources(options: {
  store: Store;
  sources: readonly GroundedDocumentSource[];
  draft: VideoDraft;
}) {
  const spaceId = options.store.activeSpaceId;
  if (!spaceId) return;
  return runWorkflowTemplate({
    spaceId,
    templateId: "video-production",
    inputNodeIds: options.sources.map(({ nodeId }) => nodeId),
    parameters: {
      script: { title: `${options.draft.title} · Script`, prompt: options.draft.prompt },
      video: {
        title: options.draft.title,
        prompt: options.draft.prompt,
        aspectRatio: options.draft.aspectRatio,
        durationSeconds: options.draft.durationSeconds,
      },
    },
    targets: { "draft-script": options.draft.textTarget, "generate-video": options.draft.target },
  });
}
