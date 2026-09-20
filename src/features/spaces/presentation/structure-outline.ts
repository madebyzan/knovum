import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { indexStructure, structureAncestors } from "../model/structure/structure-index";
import type { SpaceStructure, StructureItem } from "../model/structure/structure-schema";

export interface StructureOutlineItem {
  id: string;
  title: string;
  kind: StructureItem["kind"];
  locked: boolean;
  node?: SpaceCanvasNode;
  children?: StructureOutlineItem[];
}
export function projectStructureOutline(structure: SpaceStructure, nodes: readonly SpaceCanvasNode[]) {
  const tree = indexStructure(structure);
  const content = new Map(nodes.map((node) => [node.id, node]));
  const records = new Map<string, StructureOutlineItem>();
  for (const item of structure.items) {
    const node = item.kind === "content" ? content.get(item.nodeId) : undefined;
    if (item.kind === "content" && !node) continue;
    records.set(item.id, {
      id: item.id,
      kind: item.kind,
      title: item.kind === "content" ? node!.data.title : item.title,
      locked: item.locked || structureAncestors(tree, item.id).some((parent) => parent.locked),
      ...(node ? { node } : {}),
    });
  }
  for (const [parentId, children] of tree.children) {
    if (parentId === null) continue;
    const parent = records.get(parentId);
    const available = children.flatMap(({ id }) => (records.has(id) ? [records.get(id)!] : []));
    if (parent && available.length) parent.children = available;
  }
  return {
    records,
    roots: (tree.children.get(null) ?? []).flatMap(({ id }) => (records.has(id) ? [records.get(id)!] : [])),
  };
}
