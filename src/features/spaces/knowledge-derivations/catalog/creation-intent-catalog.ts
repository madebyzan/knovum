import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { getDocumentConversionTargets, needsDocumentRecognition } from "../document/conversion-source";
import { canRemoveImageBackground } from "../image-cutout";
import { canAddVideoSubtitles } from "../subtitled-video";
import { selectionCapabilities } from "./capabilities";
import { decideCreationIntents, type CreationIntentDecisions } from "./creation-intent-definitions";
import { groundedDocumentSources } from "./grounded-document";
import { hasDerivableKnowledge } from "./source-capability";

export function creationIntentCatalog(nodes: readonly SpaceCanvasNode[]) {
  const single = nodes.length === 1 ? nodes[0] : undefined;
  const sources = groundedDocumentSources(nodes);
  const cutout = canRemoveImageBackground(single);
  const capabilities = selectionCapabilities(nodes, sources);
  const decisions: CreationIntentDecisions = decideCreationIntents({
    selectionCount: nodes.length,
    hasKnowledgeSources: Boolean(sources),
    hasVideoSources:
      Boolean(sources) ||
      (nodes.length > 0 &&
        nodes.length <= 32 &&
        nodes.every(
          (node) =>
            Boolean(node.data.knowledgeRevision) &&
            (node.data.kind === "image"
              ? Boolean(node.data.sourceRevision) && !node.data.processingState
              : hasDerivableKnowledge(node.data)),
        )),
    imageCount: capabilities.imageCount,
    canRemoveBackground: cutout,
    canAddVideoSubtitles: canAddVideoSubtitles(single),
    documentTargets: getDocumentConversionTargets(single),
    documentRecognitionRequired: needsDocumentRecognition(single),
  });
  return {
    decisions,
    single,
    sources,
    capabilities,
  };
}
