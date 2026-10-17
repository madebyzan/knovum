import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { getDocumentConversionTargets, needsDocumentRecognition } from "../document/conversion-source";
import { canRemoveImageBackground } from "../image-cutout";
import { canSplitImageLayers } from "../image-layers";
import { canAddVideoSubtitles } from "../subtitled-video";
import { decideCreationIntents, type CreationIntentDecisions } from "./creation-intent-definitions";
import { creationSources } from "./creation-source";

export function creationIntentCatalog(nodes: readonly SpaceCanvasNode[]) {
  const single = nodes.length === 1 ? nodes[0] : undefined;
  const sources = creationSources(nodes);
  const decisions: CreationIntentDecisions = decideCreationIntents({
    selectionCount: nodes.length,
    sourceCount: sources.length,
    imageCount: nodes.filter(({ data }) => data.kind === "image" && data.sourceRevision).length,
    canRemoveBackground: canRemoveImageBackground(single),
    canSplitImageLayers: canSplitImageLayers(single),
    canAddVideoSubtitles: canAddVideoSubtitles(single),
    documentTargets: getDocumentConversionTargets(single),
    documentRecognitionRequired: needsDocumentRecognition(single),
  });
  return { decisions, single, sources };
}
