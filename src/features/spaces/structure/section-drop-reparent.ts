import { maxBy } from "es-toolkit";

import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex, effectiveOperationRoots } from "../model/structure/structure-index";
import { boundsForOperationRoots } from "../model/structure/world-bounds";
import type { SpaceEditCommand, SpacePoint } from "./space-edit-command";

const CONTAINMENT_TOLERANCE = 0.5;

export function sectionDropMoveCommand(
  snapshot: SpaceEditSnapshot,
  rootIds: readonly string[],
  delta: SpacePoint,
  allowCollapsedSectionId?: string,
): SpaceEditCommand {
  const reparent = sectionDropReparentCommand(
    snapshot,
    rootIds,
    delta,
    allowCollapsedSectionId ? { allowCollapsedSectionId } : {},
  );
  if (!reparent) return { kind: "structure.move", rootIds: [...rootIds], delta };
  return {
    kind: "structure.move-to-container",
    rootIds: [...rootIds],
    delta,
    parentId: reparent.parentId,
    order: reparent.order,
    ...(allowCollapsedSectionId === reparent.parentId ? { expandSectionId: allowCollapsedSectionId } : {}),
  };
}

export function sectionDropReparentCommand(
  snapshot: SpaceEditSnapshot,
  rootIds: readonly string[],
  delta: SpacePoint,
  options: { allowCollapsedSectionId?: string; inspectCollapsed?: boolean } = {},
): Extract<SpaceEditCommand, { kind: "structure.reparent" }> | undefined {
  const index = createStructureIndex(snapshot.structure);
  const roots = effectiveOperationRoots(rootIds, index);
  const currentParentId = reparentableParentId(roots, index.structureById);
  if (currentParentId === undefined) return undefined;
  const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const bounds = boundsForOperationRoots(
    roots.map(({ id }) => id),
    index,
    layouts,
  );
  if (!bounds) return undefined;
  const moved = {
    left: bounds.left + delta.x,
    top: bounds.top + delta.y,
    right: bounds.right + delta.x,
    bottom: bounds.bottom + delta.y,
  };
  const selected = new Set(roots.map(({ id }) => id));
  const targets = snapshot.layouts.flatMap((layout) =>
    sectionDropTarget(layout, { structureById: index.structureById, selected, moved, options }),
  );
  const target = maxBy(targets, (candidate) => candidate.paintOrder);
  const parentId = target?.id ?? null;
  if (parentId === currentParentId) return undefined;
  return {
    kind: "structure.reparent",
    rootIds: roots.map(({ id }) => id),
    parentId,
    order: (index.childrenByParent.get(parentId) ?? []).length,
  };
}

function reparentableParentId(
  roots: ReturnType<typeof effectiveOperationRoots>,
  structureById: ReturnType<typeof createStructureIndex>["structureById"],
): string | null | undefined {
  if (!roots.length || roots.some(({ kind }) => kind === "section")) return undefined;
  const parentId = roots[0]!.parentId;
  if (roots.some((root) => root.parentId !== parentId)) return undefined;
  const parent = parentId ? structureById.get(parentId) : undefined;
  return parent && parent.kind !== "section" ? undefined : parentId;
}

function sectionDropTarget(
  layout: SpaceEditSnapshot["layouts"][number],
  context: {
    structureById: ReturnType<typeof createStructureIndex>["structureById"];
    selected: ReadonlySet<string>;
    moved: { left: number; top: number; right: number; bottom: number };
    options: { allowCollapsedSectionId?: string; inspectCollapsed?: boolean };
  },
) {
  const { structureById, selected, moved, options } = context;
  const section = structureById.get(layout.structureNodeId);
  if (layout.kind !== "section" || section?.kind !== "section") return [];
  const collapsedUnavailable =
    layout.collapsed && !options.inspectCollapsed && options.allowCollapsedSectionId !== section.id;
  if (collapsedUnavailable || section.locked || selected.has(section.id)) return [];
  const frame = {
    left: layout.position.x,
    top: layout.position.y,
    right: layout.position.x + layout.size.width,
    bottom: layout.position.y + layout.size.height,
  };
  return contains(frame, moved) ? [{ id: section.id, paintOrder: section.order }] : [];
}

export function sectionDropCandidateId(
  snapshot: SpaceEditSnapshot,
  rootIds: readonly string[],
  delta: SpacePoint,
  inspectCollapsed = false,
) {
  const command = sectionDropReparentCommand(snapshot, rootIds, delta, { inspectCollapsed });
  if (!command) return undefined;
  if (command.parentId) return command.parentId;
  const index = createStructureIndex(snapshot.structure);
  const roots = effectiveOperationRoots(rootIds, index);
  const parentId = roots[0]?.parentId;
  return parentId && index.structureById.get(parentId)?.kind === "section" ? parentId : undefined;
}

function contains(
  container: { left: number; top: number; right: number; bottom: number },
  content: { left: number; top: number; right: number; bottom: number },
) {
  return (
    content.left >= container.left - CONTAINMENT_TOLERANCE &&
    content.top >= container.top - CONTAINMENT_TOLERANCE &&
    content.right <= container.right + CONTAINMENT_TOLERANCE &&
    content.bottom <= container.bottom + CONTAINMENT_TOLERANCE
  );
}
