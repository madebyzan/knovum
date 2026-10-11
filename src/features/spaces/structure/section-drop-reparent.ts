import { maxBy } from "es-toolkit";

import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";
import type { SpacePoint } from "../contracts/generated/SpacePoint";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { SECTION_HEADER_HEIGHT } from "../model/structure/section-metrics";
import { createStructureIndex, effectiveOperationRoots } from "../model/structure/structure-index";
import { boundsForOperationRoots, type WorldBounds } from "../model/structure/world-bounds";

const indexBySnapshot = new WeakMap<SpaceEditSnapshot, ReturnType<typeof buildSectionDropIndex>>();

export function sectionDropIndex(snapshot: SpaceEditSnapshot) {
  let index = indexBySnapshot.get(snapshot);
  if (!index) {
    index = buildSectionDropIndex(snapshot);
    indexBySnapshot.set(snapshot, index);
  }
  return index;
}

function buildSectionDropIndex(snapshot: SpaceEditSnapshot) {
  return {
    structure: createStructureIndex(snapshot.structure),
    layouts: new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout])),
    sections: snapshot.layouts.filter((layout) => layout.kind === "section"),
  };
}

interface SectionDropContext {
  snapshot: SpaceEditSnapshot;
  rootIds: readonly string[];
  delta: SpacePoint;
}

interface SectionDropReparentOptions {
  allowCollapsedSectionId?: string;
}

export function sectionDropMoveCommand(
  { snapshot, rootIds, delta }: SectionDropContext,
  allowCollapsedSectionId?: string,
): SpaceEditCommand {
  const reparent = sectionDropReparentCommand(
    { snapshot, rootIds, delta },
    allowCollapsedSectionId ? { allowCollapsedSectionId } : {},
  );
  if (!reparent) return { kind: "structure.move", rootIds: [...rootIds], delta };
  const targetLayout = reparent.parentId
    ? sectionDropIndex(snapshot).layouts.get(reparent.parentId)
    : undefined;
  return {
    kind: "structure.move-to-container",
    rootIds: [...rootIds],
    delta,
    parentId: reparent.parentId,
    order: reparent.order,
    ...(targetLayout?.kind === "section" && targetLayout.collapsed
      ? { expandSectionId: targetLayout.structureNodeId }
      : {}),
  };
}

export function sectionDropReparentCommand(
  { snapshot, rootIds, delta }: SectionDropContext,
  options: SectionDropReparentOptions = {},
): Extract<SpaceEditCommand, { kind: "structure.reparent" }> | undefined {
  const cached = sectionDropIndex(snapshot);
  const index = cached.structure;
  const roots = effectiveOperationRoots(rootIds, index);
  const currentParentId = reparentableParentId(roots, index.structureById);
  if (currentParentId === undefined) return undefined;
  const layouts = cached.layouts;
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
  const targets = cached.sections.flatMap((layout) =>
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

interface SectionDropTargetContext {
  structureById: ReturnType<typeof createStructureIndex>["structureById"];
  selected: ReadonlySet<string>;
  moved: WorldBounds;
  options: SectionDropReparentOptions;
}

function sectionDropTarget(
  layout: Extract<SpaceEditSnapshot["layouts"][number], { kind: "section" }>,
  context: SectionDropTargetContext,
) {
  const { structureById, selected, moved, options } = context;
  const section = structureById.get(layout.structureNodeId);
  if (section?.kind !== "section") throw new Error("Section structure is unavailable");
  const collapsed = layout.collapsed && options.allowCollapsedSectionId !== section.id;
  if (section.locked || selected.has(section.id)) return [];
  const frame = {
    left: layout.position.x,
    top: layout.position.y,
    right: layout.position.x + layout.size.width,
    bottom: layout.position.y + (collapsed ? SECTION_HEADER_HEIGHT : layout.size.height),
  };
  return containsCenter(frame, moved) ? [{ id: section.id, paintOrder: section.order }] : [];
}

export interface SectionDropPreview {
  sectionId: string;
  kind: "enter" | "leave";
}

export function sectionDropPreview(
  { snapshot, rootIds, delta }: SectionDropContext,
  options: SectionDropReparentOptions = {},
): SectionDropPreview | undefined {
  if (!sectionDropIndex(snapshot).sections.length) return undefined;
  const command = sectionDropReparentCommand({ snapshot, rootIds, delta }, options);
  if (!command) return undefined;
  if (command.parentId) return { sectionId: command.parentId, kind: "enter" };
  const cached = sectionDropIndex(snapshot);
  const index = cached.structure;
  const roots = effectiveOperationRoots(rootIds, index);
  const parentId = roots[0]?.parentId;
  return parentId && index.structureById.get(parentId)?.kind === "section"
    ? { sectionId: parentId, kind: "leave" }
    : undefined;
}

function containsCenter(container: WorldBounds, content: WorldBounds) {
  const x = (content.left + content.right) / 2;
  const y = (content.top + content.bottom) / 2;
  return x >= container.left && x <= container.right && y >= container.top && y <= container.bottom;
}
