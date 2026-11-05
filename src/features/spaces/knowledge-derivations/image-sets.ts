import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { startDerivationAction } from "./start-action";

/**
 * Derivations that publish a set of images whose size the source decides: the depth layers a
 * picture separates into, and the stickers a sheet carries. Each reserves its own output nodes once
 * the source reports the count, so neither asks for one.
 */
const IMAGE_SET_TITLES = {
  "split-image-layers": "spaces.creation.image.splitLayers",
  "split-sticker-sheet": "spaces.creation.image.sliceStickers",
} as const;

export type ImageSetActionId = keyof typeof IMAGE_SET_TITLES;

/** A set reads the published source asset, so an image still being prepared cannot be read. */
export function canDeriveImageSet(node: SpaceCanvasNode | undefined): boolean {
  return (
    node?.data.kind === "image" &&
    Boolean(node.data.assetPath && node.data.sourceRevision && node.data.knowledgeRevision)
  );
}

export async function deriveImageSet(actionId: ImageSetActionId, source: SpaceCanvasNode) {
  if (!canDeriveImageSet(source) || source.data.kind !== "image") return;
  const sourceRevision = source.data.knowledgeRevision;
  if (!sourceRevision) return;
  return startDerivationAction({
    actionId,
    sources: [{ nodeId: source.id, title: source.data.title, sourceRevision, revisionBasis: "knowledge" }],
    parameters: {},
    title: `${source.data.title} · ${i18n.global.t(IMAGE_SET_TITLES[actionId])}`.slice(0, 512),
  });
}
