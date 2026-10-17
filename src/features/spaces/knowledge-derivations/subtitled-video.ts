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

export function addVideoSubtitles(source: SpaceCanvasNode) {
  if (!canAddVideoSubtitles(source) || !source.data.knowledgeRevision) return;
  return startDerivationAction({
    actionId: "add-video-subtitles",
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        sourceRevision: source.data.knowledgeRevision,
        revisionBasis: "knowledge",
      },
    ],
    parameters: {},
    title: i18n.global.t("spaces.creation.video.subtitles.task"),
  });
}
