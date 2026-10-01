import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { useSpaceStore } from "../session/space-store";
import { startDerivationAction } from "./start-action";

export function canAddVideoSubtitles(node?: SpaceCanvasNode): boolean {
  return (
    node?.data.kind === "video" &&
    Boolean(node.data.sourceRevision && node.data.knowledgeRevision) &&
    !node.data.processingState
  );
}

export function addVideoSubtitles(store: ReturnType<typeof useSpaceStore>, source: SpaceCanvasNode) {
  if (!canAddVideoSubtitles(source) || !source.data.knowledgeRevision) return;
  return startDerivationAction({
    store,
    actionId: "add-video-subtitles",
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        role: "primary",
        sourceRevision: source.data.knowledgeRevision,
      },
    ],
    parameters: {},
    title: i18n.global.t("spaces.creation.video.subtitles.task"),
  });
}
