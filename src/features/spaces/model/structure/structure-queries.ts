import type { StructureNodeId, StructureLayout, StructureNode } from "./schema/structure";
import { ancestorsOf, type StructureIndex } from "./structure-index";

/** Resolve the complete content scope independently of folding, filtering and viewport mounting. */
export function contentNodeIdsWithin(rootIds: readonly StructureNodeId[], index: StructureIndex): string[] {
  const pending = rootIds.toReversed();
  const visited = new Set<StructureNodeId>();
  const content = new Set<string>();
  while (pending.length) {
    const id = pending.pop()!;
    if (visited.has(id)) continue;
    visited.add(id);
    const structureNode = index.structureById.get(id);
    if (!structureNode) throw new Error(`Structure structureNode not found: ${id}`);
    if (structureNode.kind === "content") {
      content.add(structureNode.nodeId);
      continue;
    }
    const children = index.childrenByParent.get(id) ?? [];
    for (let child = children.length - 1; child >= 0; child--) pending.push(children[child]!.id);
  }
  return [...content];
}

export function sectionAncestorOf(
  structureNodeId: StructureNodeId,
  index: StructureIndex,
): StructureNode | undefined {
  return ancestorsOf(structureNodeId, index).find((structureNode) => structureNode.kind === "section");
}

export function isStructureNodeEffectivelyLocked(
  structureNodeId: StructureNodeId,
  index: StructureIndex,
): boolean {
  const structureNode = index.structureById.get(structureNodeId);
  if (!structureNode) throw new Error(`Structure structureNode not found: ${structureNodeId}`);
  return structureNode.locked || ancestorsOf(structureNodeId, index).some(({ locked }) => locked);
}

/** Root-to-inner Group path required to reveal an structureNode in the editor. */
export function groupScopePathFor(
  structureNodeId: StructureNodeId,
  index: StructureIndex,
): StructureNodeId[] {
  return ancestorsOf(structureNodeId, index)
    .filter((structureNode) => structureNode.kind === "group")
    .map((structureNode) => structureNode.id);
}

/** Keep the deepest prefix of `currentScopePath` that remains valid and strictly encloses selection. */
export function reconcileGroupScopePath(
  currentScopePath: readonly StructureNodeId[],
  index: StructureIndex,
  selection?: {
    readonly nodeIds: readonly string[];
    readonly structureNodeIds: readonly StructureNodeId[];
  },
): StructureNodeId[] {
  let scopePath = validGroupScopePrefix(currentScopePath, index);
  if (!scopePath.length || !selection) return scopePath;
  const targetIds = selectedStructureIds(selection.nodeIds, selection.structureNodeIds, index);
  for (const id of targetIds) {
    const prefix = sharedGroupPrefixLength(scopePath, groupScopePathFor(id, index));
    if (prefix === 0) return [];
    scopePath = scopePath.slice(0, prefix);
  }
  return scopePath;
}

export function sameGroupScopePath(a: readonly StructureNodeId[], b: readonly StructureNodeId[]) {
  return a.length === b.length && a.every((id, idx) => id === b[idx]);
}

function validGroupScopePrefix(
  currentScopePath: readonly StructureNodeId[],
  index: StructureIndex,
): StructureNodeId[] {
  let scopePath: StructureNodeId[] = [];
  for (const id of currentScopePath) {
    if (index.structureById.get(id)?.kind !== "group") break;
    scopePath = groupScopePathFor(id, index).concat(id);
  }
  return scopePath;
}

function selectedStructureIds(
  nodeIds: readonly string[],
  structureNodeIds: readonly StructureNodeId[],
  index: StructureIndex,
): StructureNodeId[] {
  return [
    ...structureNodeIds.filter((id) => index.structureById.has(id)),
    ...nodeIds.flatMap((nodeId) => {
      const id = index.structureNodeIdByNodeId.get(nodeId);
      return id ? [id] : [];
    }),
  ];
}

function sharedGroupPrefixLength(
  scopePath: readonly StructureNodeId[],
  ancestors: readonly StructureNodeId[],
): number {
  const limit = Math.min(scopePath.length, ancestors.length);
  for (let i = 0; i < limit; i += 1) {
    if (scopePath[i] !== ancestors[i]) return i;
  }
  return limit;
}

/** Display endpoint only; knowledge relations retain their original NodeId endpoints. */
export function visibleRepresentativeForNode(
  nodeId: string,
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId> = new Set(),
): StructureNodeId | undefined {
  const structureNodeId = index.structureNodeIdByNodeId.get(nodeId);
  if (!structureNodeId) return undefined;
  const section = sectionAncestorOf(structureNodeId, index);
  const sectionLayout = section ? layoutById.get(section.id) : undefined;
  if (
    section &&
    sectionLayout?.kind === "section" &&
    sectionLayout.collapsed &&
    !temporarilyExpandedSectionIds.has(section.id)
  )
    return section.id;
  return structureNodeId;
}
