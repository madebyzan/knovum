import type { StructureItemLayout } from "../canvas/contract/structure-layout";
import type { StructureItem } from "../model/structure/structure-schema";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure } from "../organization/structure/structure-projection";
import type { StructureSnapshot } from "../organization/structure/structure-snapshot-contract";
import { parseStructureSnapshot } from "../organization/structure/structure-snapshot-validation";

interface Options {
  parentId?: string | null;
  order?: number;
  locked?: boolean;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  rotation?: number;
  scale?: number;
  collapsed?: boolean;
}
interface Entry {
  item: StructureItem;
  layout: StructureItemLayout;
}
function base(id: string, options: Options) {
  return {
    id,
    parentId: options.parentId ?? null,
    order: options.order ?? 0,
    locked: options.locked ?? false,
  };
}
function transform(options: Options) {
  return { x: options.x ?? 0, y: options.y ?? 0, rotation: options.rotation ?? 0, scale: options.scale ?? 1 };
}
export function contentFixture(id: string, options: Options = {}): Entry {
  return {
    item: { ...base(id, options), kind: "content", nodeId: id },
    layout: {
      kind: "content",
      itemId: id,
      transform: transform(options),
      sizeAuthority: "frame",
      size: { width: options.width ?? 100, height: options.height ?? 80 },
    },
  };
}
export function groupFixture(id: string, options: Options = {}): Entry {
  return {
    item: { ...base(id, options), kind: "group", title: id },
    layout: { kind: "group", itemId: id, transform: transform(options) },
  };
}
export function sectionFixture(id: string, options: Options = {}): Entry {
  return {
    item: { ...base(id, options), kind: "section", title: id },
    layout: {
      kind: "section",
      itemId: id,
      position: { x: options.x ?? 0, y: options.y ?? 0 },
      size: { width: options.width ?? 400, height: options.height ?? 300 },
      collapsed: options.collapsed ?? false,
    },
  };
}
export function structureFixture(entries: readonly Entry[] = []): StructureSnapshot {
  const snapshot = parseStructureSnapshot(
    {
      structure: { version: 1, items: entries.map(({ item }) => item) },
      layout: { version: 1, items: entries.map(({ layout }) => layout) },
    },
    new Set(entries.flatMap(({ item }) => (item.kind === "content" ? [item.nodeId] : []))),
  );
  return {
    structure: { version: 1, items: snapshot.structure.items.toSorted((a, b) => a.id.localeCompare(b.id)) },
    layout: { version: 1, items: snapshot.layout.items.toSorted((a, b) => a.itemId.localeCompare(b.itemId)) },
  };
}
export function fixtureWorld(snapshot: StructureSnapshot, id: string) {
  const transform = projectStructure(indexStructureSnapshot(snapshot)).transforms.get(id);
  if (!transform) throw new Error(`Missing fixture item ${id}`);
  return transform;
}
export function nestedStructureFixture(): StructureSnapshot {
  return structureFixture([
    sectionFixture("s", { x: 100, y: 200 }),
    groupFixture("g", { parentId: "s", x: 40, y: 60, rotation: 35, scale: 1.5 }),
    groupFixture("h", { parentId: "g", x: 12, y: 18, rotation: -20, scale: 0.8 }),
    contentFixture("a", { parentId: "h", x: 4, y: 7, rotation: 10 }),
    contentFixture("b", { parentId: "h", order: 1, x: 120, y: 15 }),
    contentFixture("c", { parentId: "g", order: 1, x: 220, y: 80, locked: true }),
    contentFixture("d", { parentId: "s", order: 1, x: 400, y: 50 }),
    contentFixture("outside", { order: 1, x: 700, y: 600 }),
  ]);
}
