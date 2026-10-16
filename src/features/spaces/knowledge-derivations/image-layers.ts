import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { startDerivationAction } from "./start-action";

/** A split reads the published source asset, so an image still being prepared cannot be split. */
export function canSplitImageLayers(node: SpaceCanvasNode | undefined): boolean {
  return (
    node?.data.kind === "image" &&
    Boolean(node.data.assetPath && node.data.sourceRevision && node.data.knowledgeRevision)
  );
}

/**
 * Splits an image into layer nodes. How many layers the image carries is discovered rather than
 * requested, so the Action reserves its output nodes itself once discovery reports the count.
 */
export async function splitImageLayers(source: SpaceCanvasNode) {
  if (!canSplitImageLayers(source) || source.data.kind !== "image") return;
  const sourceRevision = source.data.knowledgeRevision;
  if (!sourceRevision) return;
  return startDerivationAction({
    actionId: "split-image-layers",
    sources: [{ nodeId: source.id, title: source.data.title, role: "primary", sourceRevision }],
    parameters: {},
    title: `${source.data.title} · ${i18n.global.t("spaces.creation.image.splitLayers")}`.slice(0, 512),
  });
}
