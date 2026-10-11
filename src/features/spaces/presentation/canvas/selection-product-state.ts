import { isSpaceNodeOpenable } from "../../canvas/selection-actions";
import { currentSelectionRoots } from "../../canvas/selection/selection-roots";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { StructureNode } from "../../model/structure/schema/structure";
import {
  ancestorsOf,
  createStructureIndex,
  type StructureIndex,
} from "../../model/structure/structure-index";
import {
  contentNodeIdsWithin,
  isStructureNodeEffectivelyLocked,
} from "../../model/structure/structure-queries";
import { dissolveGroupAvailability, groupAvailability } from "../../structure/action-availability";

export interface CanvasSelectionProductState {
  allLocked: boolean;
  askScope: "selection";
  canConnect: boolean;
  canAsk: boolean;
  canCreateGroup: boolean;
  canCreateSection: boolean;
  canDissolveGroup: boolean;
  canFitSection: boolean;
  canDissolveSection: boolean;
  canToggleLock: boolean;
  canOpen: boolean;
  hasSectionMembership: boolean;
  content: SpaceCanvasNode[];
  structureNodeIds: string[];
  section: { id: string; collapsed: boolean } | null;
}

interface CanvasSelectionProductInput {
  nodes: readonly SpaceCanvasNode[];
  selectedIds: readonly string[];
  selectedStructureNodeIds: readonly string[];
  snapshot: SpaceEditSnapshot;
}

export function canvasSelectionProductState({
  nodes,
  selectedIds,
  selectedStructureNodeIds,
  snapshot,
}: CanvasSelectionProductInput): CanvasSelectionProductState {
  const selected = new Set(selectedIds);
  const explicitlySelectedContent = nodes.filter((node) => selected.has(node.id));
  const index = createStructureIndex(snapshot.structure);
  const structureNodeIds = currentSelectionRoots(
    explicitlySelectedContent.map(({ id }) => id),
    selectedStructureNodeIds,
    index,
  ).map(({ id }) => id);
  const selectedNodeIds = new Set(contentNodeIdsWithin(structureNodeIds, index));
  const content = nodes.filter(({ id }) => selectedNodeIds.has(id));
  const explicitlySelectedSemantic = explicitlySelectedContent.filter(isSpaceSemanticNode);
  const roots = structureNodeIds.map((id) => index.structureById.get(id)!);
  const parentId = roots[0]?.parentId;
  const parent = parentId ? index.structureById.get(parentId) : undefined;
  const section = roots.length === 1 && roots[0]?.kind === "section" ? roots[0] : undefined;
  const sectionLayout = section
    ? snapshot.layouts.find((layout) => layout.structureNodeId === section.id && layout.kind === "section")
    : undefined;
  const hasLockedRoot = structureNodeIds.some((id) => isStructureNodeEffectivelyLocked(id, index));
  const capabilities = selectionCapabilities({
    explicitlySelectedContent,
    explicitlySelectedSemanticCount: explicitlySelectedSemantic.length,
    hasLockedRoot,
    index,
    parent,
    parentId,
    roots,
    snapshot,
    structureNodeIds,
  });
  return {
    allLocked:
      structureNodeIds.length > 0 && structureNodeIds.every((id) => index.structureById.get(id)?.locked),
    askScope: "selection",
    canAsk: content.some(isSpaceSemanticNode),
    ...capabilities,
    hasSectionMembership: structureNodeIds.some((id) =>
      ancestorsOf(id, index).some((ancestor) => ancestor.kind === "section"),
    ),
    content,
    ...sectionCapabilities(section, hasLockedRoot, content.length),
    structureNodeIds,
    section: selectedSectionState(section, sectionLayout),
  };
}

function selectedSectionState(
  section: StructureNode | undefined,
  layout: SpaceEditSnapshot["layouts"][number] | undefined,
) {
  return section?.kind === "section" && layout?.kind === "section"
    ? { id: section.id, collapsed: layout.collapsed }
    : null;
}

interface SelectionCapabilityContext {
  explicitlySelectedContent: readonly SpaceCanvasNode[];
  explicitlySelectedSemanticCount: number;
  hasLockedRoot: boolean;
  index: StructureIndex;
  parent: StructureNode | undefined;
  parentId: string | null | undefined;
  roots: readonly StructureNode[];
  snapshot: SpaceEditSnapshot;
  structureNodeIds: readonly string[];
}

function selectionCapabilities(context: SelectionCapabilityContext) {
  const singleContent = context.roots.length === 1 && context.roots[0]?.kind === "content";
  const singleExplicitContent =
    context.explicitlySelectedContent.length === 1 ? context.explicitlySelectedContent[0] : undefined;
  return {
    canConnect:
      context.roots.length === 2 &&
      context.roots.every(({ kind }) => kind === "content") &&
      context.explicitlySelectedSemanticCount === 2,
    canCreateGroup: groupAvailability(context.snapshot, context.structureNodeIds).enabled,
    canDissolveGroup: dissolveGroupAvailability(context.snapshot, context.structureNodeIds).enabled,
    canToggleLock: context.structureNodeIds.every(
      (id) => !ancestorsOf(id, context.index).some(({ locked }) => locked),
    ),
    canCreateSection: canCreateSection(context),
    canOpen: Boolean(singleContent && singleExplicitContent && isSpaceNodeOpenable(singleExplicitContent)),
  };
}

function canCreateSection(context: SelectionCapabilityContext) {
  return (
    context.roots.length > 0 &&
    !context.hasLockedRoot &&
    context.roots.every((node) => node.kind !== "section" && node.parentId === context.parentId) &&
    (context.parentId === null || context.parent?.kind === "section")
  );
}

function sectionCapabilities(section: StructureNode | undefined, locked: boolean, contentCount: number) {
  const editable = Boolean(section) && !locked;
  return { canFitSection: editable && contentCount > 0, canDissolveSection: editable };
}
