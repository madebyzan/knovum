import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { canRemoveImageBackground } from "../image-cutout";
import { canCreateSubtitleResource } from "../subtitle-extraction";
import { subtitleVideoInputs } from "../subtitled-video";
import { selectionCapabilities } from "./capabilities";
import { decideCreationIntents, type CreationIntentDecisions } from "./creation-intent-definitions";
import { groundedDocumentSources } from "./grounded-document";

export type { CreationIntentDecisions, CreationIntentId } from "./creation-intent-definitions";

export function creationIntentCatalog(nodes: readonly SpaceCanvasNode[]) {
  const single = nodes.length === 1 ? nodes[0] : undefined;
  const sources = groundedDocumentSources(nodes);
  const cutout = canRemoveImageBackground(single);
  const subtitleResource = canCreateSubtitleResource(single);
  const subtitledVideo = subtitleVideoInputs(nodes);
  const capabilities = selectionCapabilities(nodes, sources);
  const decisions: CreationIntentDecisions = decideCreationIntents({
    selectionCount: nodes.length,
    hasKnowledgeSources: Boolean(sources),
    imageCount: capabilities.imageCount,
    canRemoveBackground: cutout,
    canExtractSubtitles: subtitleResource,
    canBurnSubtitles: Boolean(subtitledVideo),
    canCreateMeetingPackage: subtitleResource && Boolean(sources),
    canExportDocument: Boolean(single?.data.kind === "markdown" && single.data.knowledgeRevision),
  });
  return {
    decisions,
    single,
    sources,
    subtitledVideo,
    capabilities,
  };
}
