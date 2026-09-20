import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackagedStructure(manifest: SpacePackageManifest) {
  const document = manifest.canvasState.document;
  const contentIds = new Set(
    document.structure.items.flatMap((item) => (item.kind === "content" ? [item.nodeId] : [])),
  );
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  if (
    nodeIds.size !== manifest.nodes.length ||
    nodeIds.size !== contentIds.size ||
    [...nodeIds].some((id) => !contentIds.has(id))
  ) {
    throw new Error("Packaged content and structure must match exactly");
  }
  const itemIds = new Set(document.structure.items.map((item) => item.id));
  for (const scene of manifest.scenes) {
    if (scene.focusNodeIds.some((id) => !itemIds.has(id)))
      throw new Error("Packaged Saved View references an unavailable structure item");
  }
  const relations = new Set(manifest.relations.map((relation) => relation.id));
  if ((document.relationStyles ?? []).some((style) => !relations.has(style.relationId))) {
    throw new Error("Packaged relationship presentation references an unavailable relationship");
  }
}
