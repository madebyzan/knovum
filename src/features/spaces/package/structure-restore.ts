import type { StructureLayout, StructureNode } from "../model/structure/structure-contract";
import { PACKAGE_ASSET_PREFIX } from "./contract";
import { requiredMap } from "./id-maps";
import type { SpacePackageRestoreContext } from "./restore-context";
import type { SpacePackageRestoreRequest } from "./restore-transport";

export function restorePackageStructure(
  context: SpacePackageRestoreContext,
): Pick<SpacePackageRestoreRequest, "structureNodes" | "layouts" | "relationStyles"> {
  return {
    structureNodes: context.manifest.structureNodes.map((structureNode) =>
      remapStructureNode(context, structureNode),
    ),
    layouts: context.manifest.layouts.map((layout) => remapLayout(context, layout)),
    relationStyles: context.manifest.relationStyles.map((style) => ({
      ...style,
      relationId: requiredMap(context.maps.relations, style.relationId, "relation style"),
    })),
  };
}

function remapStructureNode(
  context: SpacePackageRestoreContext,
  structureNode: StructureNode,
): StructureNode {
  const id = requiredMap(context.maps.structureNodes, structureNode.id, "structure node");
  const parentId =
    structureNode.parentId === null
      ? null
      : requiredMap(context.maps.structureNodes, structureNode.parentId, "structure parent");
  if (structureNode.kind === "content") {
    return {
      ...structureNode,
      id,
      parentId,
      nodeId: requiredMap(context.maps.nodes, structureNode.nodeId, "content node"),
    };
  }
  return { ...structureNode, id, parentId };
}

function remapLayout(context: SpacePackageRestoreContext, layout: StructureLayout): StructureLayout {
  const structureNodeId = requiredMap(
    context.maps.structureNodes,
    layout.structureNodeId,
    "structure layout",
  );
  if (layout.kind === "group") return { ...layout, structureNodeId };
  const appearance = layout.appearance ? { ...layout.appearance } : undefined;
  if (appearance?.background?.startsWith(PACKAGE_ASSET_PREFIX)) {
    const token = appearance.background.slice(PACKAGE_ASSET_PREFIX.length);
    const path = context.assets.get(token);
    if (!path) throw new Error("Package structure references an unknown asset");
    appearance.background = path;
  }
  return { ...layout, structureNodeId, ...(appearance ? { appearance } : {}) };
}
