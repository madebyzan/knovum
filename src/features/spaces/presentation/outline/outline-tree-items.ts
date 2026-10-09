import type { StructureNodeId } from "../../model/structure/schema/structure";
import type { StructureIndex } from "../../model/structure/structure-index";

/** A type alias rather than an interface: TreeRoot constrains its item generic to Record<string, any>. */
export type OutlineTreeItem = {
  children?: OutlineTreeItem[];
  id: StructureNodeId;
  kind: "content" | "group" | "section";
  locked: boolean;
  title: string;
};

interface OutlineTreeContext {
  titleByNodeId: ReadonlyMap<string, string>;
  index: StructureIndex;
}

/** Omitting `children` on a leaf is what stops TreeItem from treating it as expandable. */
export function buildOutlineTreeItems(
  index: StructureIndex,
  titleByNodeId: ReadonlyMap<string, string>,
): OutlineTreeItem[] {
  return outlineLevel(null, { titleByNodeId, index }, false);
}

function outlineLevel(
  parentId: StructureNodeId | null,
  context: OutlineTreeContext,
  inheritedLocked: boolean,
): OutlineTreeItem[] {
  return (context.index.childrenByParent.get(parentId) ?? []).map((structureNode) => {
    const title =
      structureNode.kind === "content"
        ? context.titleByNodeId.get(structureNode.nodeId)
        : structureNode.title;
    if (title === undefined) throw new Error(`Content title missing for ${structureNode.id}`);
    const locked = inheritedLocked || structureNode.locked;
    const item: OutlineTreeItem = { id: structureNode.id, kind: structureNode.kind, locked, title };
    const children = outlineLevel(structureNode.id, context, locked);
    if (children.length) item.children = children;
    return item;
  });
}
