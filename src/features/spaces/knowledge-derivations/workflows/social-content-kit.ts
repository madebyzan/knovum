import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { groundedDocumentSources } from "../catalog/grounded-document";
import { createGroundedDocument } from "../grounded-document";
import { executePersistedWorkflow } from "../orchestration/persisted-workflow";
import { loadWorkflow } from "../orchestration/workflow-registry";
import { createVisualization } from "../visualization";
import type { ImageAspectRatio } from "../visualization";
import { nodeStepOutput } from "./output";

type Store = ReturnType<typeof useSpaceStore>;
type Action = "write-social-copy" | "create-companion-image";
export type SocialPlatform = "generic" | "xiaohongshu" | "linkedin";
export type SocialTone = "professional" | "friendly" | "energetic";

export interface SocialContentKitDraft {
  title: string;
  platform: SocialPlatform;
  audience: string;
  goal: string;
  tone: SocialTone;
  imageStyle: string;
  aspectRatio: ImageAspectRatio;
  textTarget?: ExecutionTarget | undefined;
  imageTarget?: ExecutionTarget | undefined;
}

interface SocialContentKitContext {
  store: Store;
  selected: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  draft: SocialContentKitDraft;
}

export async function createSocialContentKit(context: SocialContentKitContext) {
  const { store, selected, sources, draft } = context;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const workflow = await loadWorkflow<Action>("social-content-kit");
  return executePersistedWorkflow<Action, SpaceCanvasNode>({
    spaceId,
    workflow,
    task: { title: i18n.global.t("spaces.creation.socialContentKit"), detail: draft.title },
    adapters: {
      "write-social-copy": async () => {
        const completed = await createGroundedDocument({
          store,
          selected,
          sources,
          draft: {
            title: draft.title,
            purpose: socialCopyPurpose(draft),
            target: draft.textTarget,
          },
        });
        const output = completed && store.nodes.find(({ id }) => id === completed.nodeId);
        if (!output) throw new Error("Social copy was not created");
        return nodeStepOutput(output);
      },
      "create-companion-image": async ({ inputs }) => {
        const copy = inputs.copy?.output;
        const copySource = copy && groundedDocumentSources([copy])?.[0];
        if (!copy || !copySource) throw new Error("Social copy is not ready for visualization");
        const imageSources: GroundedDocumentSource[] = [
          copySource,
          ...sources.map((source): GroundedDocumentSource => ({
            nodeId: source.nodeId,
            title: source.title,
            role: "source",
            sourceRevision: source.sourceRevision,
          })),
        ];
        const completed = await createVisualization({
          store,
          selected: [...selected, copy],
          sources: imageSources,
          draft: {
            title: `${draft.title} · Visual`,
            intent: socialImageIntent(draft),
            style: draft.imageStyle,
            aspectRatio: draft.aspectRatio,
            target: draft.imageTarget,
          },
        });
        const output = completed && store.nodes.find(({ id }) => id === completed.nodeId);
        if (!output) throw new Error("Social companion image was not created");
        return nodeStepOutput(output);
      },
    },
  });
}

function socialCopyPurpose(draft: SocialContentKitDraft) {
  return [
    `Create publication-ready ${draft.platform} social copy.`,
    `Audience: ${draft.audience.trim()}.`,
    `Communication goal: ${draft.goal.trim()}.`,
    `Tone: ${draft.tone}.`,
    "Preserve factual grounding in the selected knowledge and do not invent claims.",
  ].join("\n");
}

function socialImageIntent(draft: SocialContentKitDraft) {
  return [
    `Create the companion visual for the generated ${draft.platform} post.`,
    `Audience: ${draft.audience.trim()}.`,
    `Goal: ${draft.goal.trim()}.`,
    "Reflect the post's central idea without placing long text in the image.",
  ].join("\n");
}
