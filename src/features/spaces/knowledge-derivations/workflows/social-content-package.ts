import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { useSpaceStore } from "../../session/space-store";
import { startBuiltinWorkflow } from "../../workflows/run-builtin-workflow";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import type { ImageAspectRatio } from "../visualization";

type Store = ReturnType<typeof useSpaceStore>;
export type SocialPlatform = "generic" | "xiaohongshu" | "linkedin";
export type SocialTone = "professional" | "friendly" | "energetic";
export interface SocialContentPackageDraft {
  title: string;
  platform: SocialPlatform;
  audience: string;
  goal: string;
  tone: SocialTone;
  imageStyle: string;
  aspectRatio: ImageAspectRatio;
  textTarget?: ExecutionTarget;
  imageTarget?: ExecutionTarget;
}

export async function createSocialContentPackage(context: {
  store: Store;
  sources: readonly GroundedDocumentSource[];
  draft: SocialContentPackageDraft;
}) {
  const spaceId = context.store.activeSpaceId;
  if (!spaceId) return;
  const { draft } = context;
  return startBuiltinWorkflow({
    spaceId,
    templateId: "social-content-package",
    inputNodeIds: context.sources.map(({ nodeId }) => nodeId),
    parameters: {
      copy: {
        title: draft.title,
        platform: draft.platform,
        audience: draft.audience,
        goal: draft.goal,
        tone: draft.tone,
      },
      image: {
        title: `${draft.title} · Visual`,
        imageStyle: draft.imageStyle,
        aspectRatio: draft.aspectRatio,
      },
    },
    targets: { "draft-social-post": draft.textTarget, "generate-image": draft.imageTarget },
  });
}
