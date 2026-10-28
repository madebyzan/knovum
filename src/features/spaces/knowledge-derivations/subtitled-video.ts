import type { AiRequirement } from "../../../foundation/ai/ai-readiness";
import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { i18n } from "../../../foundation/i18n/instance";
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

export async function addVideoSubtitles(source: SpaceCanvasNode) {
  if (!canAddVideoSubtitles(source) || !source.data.knowledgeRevision) return;
  const sourceRevision = source.data.knowledgeRevision;
  if (!(await admitAiRequirements(subtitleRequirements(source)))) return;
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
    parameters: {},
    title: i18n.global.t("spaces.creation.video.subtitles.task"),
  });
}
