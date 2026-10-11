import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deriveImage } from "./image-operation";

export function canRemoveImageBackground(node: SpaceCanvasNode | undefined): boolean {
  return (
    node?.data.kind === "image" &&
    Boolean(node.data.assetPath && node.data.sourceRevision && node.data.knowledgeRevision)
  );
}

export async function createImageCutout(source: SpaceCanvasNode) {
  if (!canRemoveImageBackground(source)) return;
  return deriveImage(source, { kind: "remove-background" });
}
