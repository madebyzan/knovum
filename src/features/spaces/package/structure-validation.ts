import { validateStructureSnapshot } from "../model/structure/structure-validation";
import { PACKAGE_ASSET_PREFIX } from "./contract";
import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackageStructure(manifest: SpacePackageManifest) {
  validateStructureSnapshot(manifest.structureNodes, manifest.layouts);
  validateContentCoverage(manifest);
  validateRelationStyles(manifest);
  validateLayoutAssets(manifest);
}

function validateLayoutAssets(manifest: SpacePackageManifest) {
  for (const layout of manifest.layouts) {
    if (layout.kind === "group" || layout.appearance?.background === undefined) continue;
    if (!layout.appearance.background.startsWith(PACKAGE_ASSET_PREFIX))
      throw new Error(`Structure layout background is not a packaged asset: ${layout.structureNodeId}`);
  }
}

function validateContentCoverage(manifest: SpacePackageManifest) {
  const nodeIds = new Set(manifest.nodes.map(({ id }) => id));
  const contentNodeIds = new Set<string>();
  for (const structureNode of manifest.structureNodes) {
    if (structureNode.kind !== "content") continue;
    if (!nodeIds.has(structureNode.nodeId))
      throw new Error(`Structure references an unknown node: ${structureNode.nodeId}`);
    contentNodeIds.add(structureNode.nodeId);
  }
  if (contentNodeIds.size !== nodeIds.size)
    throw new Error("Package structure must contain every active content node exactly once");
}

function validateRelationStyles(manifest: SpacePackageManifest) {
  const eligible = new Set(
    manifest.relations.filter(({ status }) => status !== "invalidated").map(({ id }) => id),
  );
  const styled = new Set<string>();
  for (const style of manifest.relationStyles) {
    if (styled.has(style.relationId)) throw new Error(`Duplicate relation style: ${style.relationId}`);
    if (!eligible.has(style.relationId))
      throw new Error(`Relation style references an unavailable relation: ${style.relationId}`);
    styled.add(style.relationId);
  }
}
