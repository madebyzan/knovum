import type { AiRequirement } from "../../../foundation/ai/ai-readiness";
import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { preferredLocalModel } from "../../../foundation/ai/local-model-choice";
import { i18n } from "../../../foundation/i18n/instance";
import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { startDerivationAction } from "./start-action";

export function canAddVideoSubtitles(node?: SpaceCanvasNode): boolean {
  return (
    node?.data.kind === "video" &&
    Boolean(node.data.sourceRevision && node.data.knowledgeRevision) &&
    !node.data.processingState
  );
}

/** Subtitles reuse a transcript the video already has; without one the action transcribes it first. */
function subtitleRequirements(source: SpaceCanvasNode): readonly AiRequirement[] {
  return source.data.kind === "video" && source.data.mediaKnowledge
    ? []
    : [{ kind: "localCapability", capability: "transcription" }];
}

export async function addVideoSubtitles(source: SpaceCanvasNode, transcriptionModel?: LocalModelId) {
  if (!canAddVideoSubtitles(source) || !source.data.knowledgeRevision) return;
  const sourceRevision = source.data.knowledgeRevision;
  if (!(await admitAiRequirements(subtitleRequirements(source)))) return;
  const model = transcriptionModel ?? (await preferredLocalModel("transcription"));
  if (!model) return;
  return startDerivationAction({
    actionId: "add-video-subtitles",
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        sourceRevision,
        revisionBasis: "knowledge",
      },
    ],
    parameters: { transcriptionModel: model },
    title: i18n.global.t("spaces.creation.video.subtitles.task"),
  });
}
