import { sortBy } from "es-toolkit";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { JsonCanvasImportPlanV1 } from "./json-canvas-import-contract";

export function mergeImportedJsonCanvasStructure(
  snapshot: SpaceEditSnapshot,
  plan: JsonCanvasImportPlanV1,
  identities: {
    contentIds: ReadonlyMap<string, string>;
    sectionIds: ReadonlyMap<string, string>;
    backgroundPaths: ReadonlyMap<string, string>;
    defaultSectionTitle: string;
  },
): SpaceEditSnapshot {
  const structure = [...snapshot.structure];
  const layouts = [...snapshot.layouts];
  const nextRootOrder = structure.filter(({ parentId }) => parentId === null).length;
  const rootSources = sortBy(
    [
      ...plan.sections.map((section) => ({ sourceId: section.sourceId, order: section.order })),
      ...plan.contents
        .filter(({ parentSectionSourceId }) => !parentSectionSourceId)
        .map((content) => ({ sourceId: content.sourceId, order: content.order })),
    ],
    [({ order }) => order],
  );
  const rootOrders = new Map(rootSources.map(({ sourceId }, offset) => [sourceId, nextRootOrder + offset]));
  const childOrders = new Map<string, number>();
  for (const section of plan.sections) appendSection(section);
  for (const content of plan.contents) appendContent(content);
  return { ...snapshot, structure, layouts };

  function appendSection(section: JsonCanvasImportPlanV1["sections"][number]) {
    const id = requiredIdentity(identities.sectionIds, section.sourceId, "Section");
    const background = identities.backgroundPaths.get(section.sourceId);
    structure.push({
      id,
      kind: "section",
      parentId: null,
      order: requiredOrder(rootOrders, section.sourceId),
      locked: false,
      title: section.title?.trim() || identities.defaultSectionTitle,
    });
    layouts.push({
      structureNodeId: id,
      kind: "section",
      position: { x: section.frame.x, y: section.frame.y },
      size: { width: section.frame.width, height: section.frame.height },
      collapsed: false,
      ...(section.appearance
        ? { appearance: { ...section.appearance, ...(background ? { background } : {}) } }
        : {}),
    });
  }

  function appendContent(content: JsonCanvasImportPlanV1["contents"][number]) {
    const id = requiredIdentity(identities.contentIds, content.sourceId, "content");
    const parent = importedContentParent(content);
    const parentId = parent?.id ?? null;
    const order = contentOrder(content.sourceId, parentId);
    if (parentId !== null) childOrders.set(parentId, order + 1);
    structure.push({ id, kind: "content", parentId, order, locked: false, nodeId: id });
    layouts.push({
      structureNodeId: id,
      kind: "content",
      transform: {
        x: content.frame.x - (parent?.frame.x ?? 0),
        y: content.frame.y - (parent?.frame.y ?? 0),
        rotation: 0,
        scale: 1,
      },
      size: { width: content.frame.width, height: content.frame.height },
      sizeAuthority: "frame",
      ...(content.appearance ? { appearance: content.appearance } : {}),
      ...(content.fileSubpath ? { fileSubpath: content.fileSubpath } : {}),
    });
  }

  function importedContentParent(content: JsonCanvasImportPlanV1["contents"][number]) {
    if (!content.parentSectionSourceId) return undefined;
    const id = requiredIdentity(identities.sectionIds, content.parentSectionSourceId, "parent Section");
    const section = plan.sections.find(({ sourceId }) => sourceId === content.parentSectionSourceId);
    if (!section)
      throw new Error(`JSON Canvas parent Section is unavailable: ${content.parentSectionSourceId}`);
    return { id, frame: section.frame };
  }

  function contentOrder(sourceId: string, parentId: string | null) {
    return parentId === null ? requiredOrder(rootOrders, sourceId) : (childOrders.get(parentId) ?? 0);
  }
}

function requiredIdentity(ids: ReadonlyMap<string, string>, sourceId: string, kind: string) {
  const id = ids.get(sourceId);
  if (!id) throw new Error(`JSON Canvas ${kind} identity is unavailable: ${sourceId}`);
  return id;
}

function requiredOrder(orders: ReadonlyMap<string, number>, sourceId: string) {
  const order = orders.get(sourceId);
  if (order === undefined) throw new Error(`JSON Canvas root order is unavailable: ${sourceId}`);
  return order;
}
