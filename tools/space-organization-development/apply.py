"""Reviewed current-schema building blocks for the ongoing product cutover, not a runtime migration."""

from pathlib import Path
import subprocess

ROOT = Path("src/features/spaces")
FILES = {
"model/structure/structure-schema.ts": '''import { z } from "zod";

export const MAX_STRUCTURE_ITEMS = 100_000;
export const MAX_STRUCTURE_DEPTH = 64;
export const itemIdSchema = z.string().min(1).max(128).regex(/^\\S+$/u);
export const containerTitleSchema = z.string().min(1).max(512).refine((title) => title === title.trim());
const base = {
  id: itemIdSchema,
  parentId: itemIdSchema.nullable(),
  order: z.number().int().nonnegative().max(MAX_STRUCTURE_ITEMS - 1),
  locked: z.boolean(),
};
export const structureItemSchema = z.discriminatedUnion("kind", [
  z.object({ ...base, kind: z.literal("content"), nodeId: itemIdSchema }).strict(),
  z.object({ ...base, kind: z.literal("group"), title: containerTitleSchema }).strict(),
  z.object({ ...base, kind: z.literal("section"), title: containerTitleSchema }).strict(),
]);
export const spaceStructureSchema = z.object({
  version: z.literal(1), items: z.array(structureItemSchema).max(MAX_STRUCTURE_ITEMS),
}).strict();
export type StructureItem = z.infer<typeof structureItemSchema>;
export type SpaceStructure = z.infer<typeof spaceStructureSchema>;
''',
"model/structure/structure-error.ts": '''export type StructureErrorCode =
  | "missing-item" | "duplicate-identity" | "invalid-parent" | "invalid-order"
  | "cycle" | "depth-limit" | "empty-group" | "invalid-layout" | "invalid-reference"
  | "invalid-selection" | "different-parents" | "locked" | "history-conflict";

export class StructureError extends Error {
  constructor(readonly code: StructureErrorCode, message: string, readonly itemIds: readonly string[] = []) {
    super(message);
    this.name = "StructureError";
  }
}
''',
"model/structure/structure-index.ts": '''import { StructureError } from "./structure-error";
import type { SpaceStructure, StructureItem } from "./structure-schema";

export interface StructureIndex {
  readonly items: ReadonlyMap<string, StructureItem>;
  readonly children: ReadonlyMap<string | null, readonly StructureItem[]>;
}
export function indexStructure(structure: SpaceStructure): StructureIndex {
  const items = new Map<string, StructureItem>();
  const children = new Map<string | null, StructureItem[]>();
  for (const item of structure.items) {
    if (items.has(item.id)) throw new StructureError("duplicate-identity", "Structure identities must be unique", [item.id]);
    items.set(item.id, item);
    const siblings = children.get(item.parentId) ?? [];
    siblings.push(item);
    children.set(item.parentId, siblings);
  }
  for (const [parent, siblings] of children) children.set(parent, siblings.toSorted((a, b) => a.order - b.order));
  return { items, children };
}
export function requireStructureItem(index: StructureIndex, id: string): StructureItem {
  const item = index.items.get(id);
  if (!item) throw new StructureError("missing-item", "The structure item is unavailable", [id]);
  return item;
}
/** Queries accept topology validated at the document boundary. */
export function structureAncestors(index: StructureIndex, id: string): StructureItem[] {
  const ancestors: StructureItem[] = [];
  let parentId = requireStructureItem(index, id).parentId;
  while (parentId !== null) {
    const parent = requireStructureItem(index, parentId);
    ancestors.push(parent);
    parentId = parent.parentId;
  }
  return ancestors;
}
export function structureSubtree(index: StructureIndex, roots: readonly string[]): StructureItem[] {
  const pending = roots.toReversed();
  const seen = new Set<string>();
  const result: StructureItem[] = [];
  while (pending.length) {
    const id = pending.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    const item = requireStructureItem(index, id);
    result.push(item);
    for (const child of (index.children.get(id) ?? []).toReversed()) pending.push(child.id);
  }
  return result;
}
''',
"model/structure/structure-validation.ts": '''import { StructureError } from "./structure-error";
import type { StructureIndex } from "./structure-index";
import { MAX_STRUCTURE_DEPTH, type StructureItem } from "./structure-schema";

export function validateStructure(index: StructureIndex) {
  const contentIds = new Set<string>();
  for (const item of index.items.values()) {
    validateParent(index, item);
    if (item.kind === "content") {
      if (item.id !== item.nodeId || contentIds.has(item.nodeId)) {
        throw new StructureError("invalid-reference", "Each content identity has exactly one placement using its own identity", [item.id]);
      }
      contentIds.add(item.nodeId);
    }
    if (item.kind === "group" && !index.children.get(item.id)?.length) {
      throw new StructureError("empty-group", "An empty Group cannot be committed", [item.id]);
    }
  }
  validateOrder(index);
  validateDepth(index);
}
function validateParent(index: StructureIndex, item: StructureItem) {
  if (item.kind === "section" && item.parentId !== null) {
    throw new StructureError("invalid-parent", "Sections belong to the Space root", [item.id]);
  }
  if (item.parentId !== null) {
    const parent = index.items.get(item.parentId);
    if (!parent || parent.kind === "content") throw new StructureError("invalid-parent", "A parent must be an existing container", [item.id]);
  }
}
function validateOrder(index: StructureIndex) {
  for (const siblings of index.children.values()) for (const [order, item] of siblings.entries()) {
    if (item.order !== order) throw new StructureError("invalid-order", "Sibling order must be unique and contiguous", [item.id]);
  }
}
function validateDepth(index: StructureIndex) {
  const pending = (index.children.get(null) ?? []).map((item) => ({ id: item.id, depth: 1 }));
  let visited = 0;
  while (pending.length) {
    const { id, depth } = pending.pop()!;
    if (depth > MAX_STRUCTURE_DEPTH) throw new StructureError("depth-limit", "Structure nesting exceeds its limit", [id]);
    visited += 1;
    for (const child of index.children.get(id) ?? []) pending.push({ id: child.id, depth: depth + 1 });
  }
  if (visited !== index.items.size) throw new StructureError("cycle", "Structure contains a parent cycle");
}
export function validateContentPlacements(index: StructureIndex, nodeIds: ReadonlySet<string>) {
  const missing = new Set(nodeIds);
  for (const item of index.items.values()) {
    if (item.kind === "content" && !missing.delete(item.nodeId)) {
      throw new StructureError("invalid-reference", "A placement must reference active content in this Space", [item.id]);
    }
  }
  if (missing.size) throw new StructureError("invalid-reference", "Every active content item requires a placement", [...missing]);
}
''',
"canvas/contract/structure-layout.ts": '''import { z } from "zod";
import { itemIdSchema, MAX_STRUCTURE_ITEMS } from "../../model/structure/structure-schema";
import { canvasNodeAppearanceSchema } from "./canvas-document";

export const WORLD_COORDINATE_LIMIT = 1_000_000_000_000;
export const WORLD_SCALE_MIN = 0.000001;
export const WORLD_SCALE_MAX = 1_000_000;
const coordinate = z.number().finite().min(-WORLD_COORDINATE_LIMIT).max(WORLD_COORDINATE_LIMIT);
export const structurePointSchema = z.object({ x: coordinate, y: coordinate }).strict();
export const structureSizeSchema = z.object({
  width: z.number().finite().positive().max(WORLD_COORDINATE_LIMIT),
  height: z.number().finite().positive().max(WORLD_COORDINATE_LIMIT),
}).strict();
export const structureTransformSchema = z.object({
  x: coordinate, y: coordinate, rotation: z.number().finite().min(-180).lt(180),
  scale: z.number().finite().min(WORLD_SCALE_MIN).max(WORLD_SCALE_MAX),
}).strict();
export const structureItemLayoutSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("content"), itemId: itemIdSchema, transform: structureTransformSchema,
    size: structureSizeSchema, sizeAuthority: z.enum(["content", "frame"]),
    appearance: canvasNodeAppearanceSchema.optional(), fileSubpath: z.string().startsWith("#").optional(),
  }).strict(),
  z.object({ kind: z.literal("group"), itemId: itemIdSchema, transform: structureTransformSchema }).strict(),
  z.object({ kind: z.literal("section"), itemId: itemIdSchema, position: structurePointSchema,
    size: structureSizeSchema, collapsed: z.boolean(), appearance: canvasNodeAppearanceSchema.optional(),
  }).strict(),
]);
export const structureLayoutSchema = z.object({
  version: z.literal(1), items: z.array(structureItemLayoutSchema).max(MAX_STRUCTURE_ITEMS),
}).strict();
export type StructurePoint = z.infer<typeof structurePointSchema>;
export type StructureSize = z.infer<typeof structureSizeSchema>;
export type StructureTransform = z.infer<typeof structureTransformSchema>;
export type StructureItemLayout = z.infer<typeof structureItemLayoutSchema>;
export type StructureLayout = z.infer<typeof structureLayoutSchema>;
export type StructureBounds = StructurePoint & StructureSize;
''',
"canvas/document/structure-transform.ts": '''import type { StructureBounds, StructureItemLayout, StructurePoint, StructureSize, StructureTransform } from "../contract/structure-layout";

export const IDENTITY_STRUCTURE_TRANSFORM: Readonly<StructureTransform> = Object.freeze({ x: 0, y: 0, rotation: 0, scale: 1 });
export function normalizeStructureRotation(degrees: number) {
  const wrapped = ((degrees + 180) % 360 + 360) % 360 - 180;
  return Object.is(wrapped, -0) ? 0 : wrapped;
}
export function transformStructurePoint(transform: StructureTransform, point: StructurePoint): StructurePoint {
  const radians = transform.rotation * Math.PI / 180;
  const cosine = Math.cos(radians) * transform.scale;
  const sine = Math.sin(radians) * transform.scale;
  return { x: transform.x + cosine * point.x - sine * point.y, y: transform.y + sine * point.x + cosine * point.y };
}
export function composeStructureTransforms(parent: StructureTransform, local: StructureTransform): StructureTransform {
  return { ...transformStructurePoint(parent, local), rotation: normalizeStructureRotation(parent.rotation + local.rotation), scale: parent.scale * local.scale };
}
export function invertStructureTransform(transform: StructureTransform): StructureTransform {
  const linear = { x: 0, y: 0, rotation: -transform.rotation, scale: 1 / transform.scale };
  return { ...linear, ...transformStructurePoint(linear, { x: -transform.x, y: -transform.y }) };
}
export function structureLayoutTransform(layout: StructureItemLayout): StructureTransform {
  return layout.kind === "section" ? { ...layout.position, rotation: 0, scale: 1 } : layout.transform;
}
export function transformedStructureBounds(transform: StructureTransform, size: StructureSize): StructureBounds {
  const points = [{ x: 0, y: 0 }, { x: size.width, y: 0 }, { x: 0, y: size.height }, { x: size.width, y: size.height }].map((point) => transformStructurePoint(transform, point));
  const x = Math.min(...points.map((point) => point.x));
  const y = Math.min(...points.map((point) => point.y));
  return { x, y, width: Math.max(...points.map((point) => point.x)) - x, height: Math.max(...points.map((point) => point.y)) - y };
}
export function unionStructureBounds(bounds: readonly StructureBounds[]): StructureBounds | undefined {
  if (!bounds.length) return undefined;
  let left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
  for (const rect of bounds) {
    left = Math.min(left, rect.x); top = Math.min(top, rect.y);
    right = Math.max(right, rect.x + rect.width); bottom = Math.max(bottom, rect.y + rect.height);
  }
  return { x: left, y: top, width: right - left, height: bottom - top };
}
''',
"organization/structure/structure-snapshot-contract.ts": '''import { z } from "zod";
import { spaceStructureSchema } from "../../model/structure/structure-schema";
import { structureLayoutSchema } from "../../canvas/contract/structure-layout";

export const structureSnapshotSchema = z.object({ structure: spaceStructureSchema, layout: structureLayoutSchema }).strict();
export type StructureSnapshot = z.infer<typeof structureSnapshotSchema>;
''',
"organization/structure/structure-context.ts": '''import type { StructureItemLayout } from "../../canvas/contract/structure-layout";
import { StructureError } from "../../model/structure/structure-error";
import { indexStructure, type StructureIndex } from "../../model/structure/structure-index";
import type { StructureSnapshot } from "./structure-snapshot-contract";

export interface StructureContext { readonly tree: StructureIndex; readonly layouts: ReadonlyMap<string, StructureItemLayout> }
export function indexStructureSnapshot(snapshot: StructureSnapshot): StructureContext {
  const tree = indexStructure(snapshot.structure);
  const layouts = new Map<string, StructureItemLayout>();
  for (const layout of snapshot.layout.items) {
    if (layouts.has(layout.itemId)) throw new StructureError("duplicate-identity", "Layout identities must be unique", [layout.itemId]);
    if (tree.items.get(layout.itemId)?.kind !== layout.kind) throw new StructureError("invalid-layout", "Layout kind must match an existing structure item", [layout.itemId]);
    layouts.set(layout.itemId, layout);
  }
  for (const item of tree.items.values()) if (!layouts.has(item.id)) throw new StructureError("invalid-layout", "Every structure item requires one layout", [item.id]);
  return { tree, layouts };
}
export function requireStructureLayout(context: StructureContext, id: string) {
  const layout = context.layouts.get(id);
  if (!layout) throw new StructureError("invalid-layout", "Item layout is unavailable", [id]);
  return layout;
}
''',
"organization/structure/structure-projection.ts": '''import type { StructureBounds, StructureTransform } from "../../canvas/contract/structure-layout";
import { composeStructureTransforms, IDENTITY_STRUCTURE_TRANSFORM, invertStructureTransform,
  structureLayoutTransform, transformedStructureBounds, unionStructureBounds } from "../../canvas/document/structure-transform";
import { requireStructureItem, structureSubtree } from "../../model/structure/structure-index";
import { requireStructureLayout, type StructureContext } from "./structure-context";

export interface StructureProjection {
  readonly transforms: ReadonlyMap<string, StructureTransform>;
  readonly bounds: ReadonlyMap<string, StructureBounds>;
  readonly paintOrder: ReadonlyMap<string, number>;
}
export function projectStructure(context: StructureContext): StructureProjection {
  const ordered = structureSubtree(context.tree, (context.tree.children.get(null) ?? []).map((item) => item.id));
  const transforms = new Map<string, StructureTransform>();
  const bounds = new Map<string, StructureBounds>();
  const paintOrder = new Map<string, number>();
  for (const [order, item] of ordered.entries()) {
    const layout = requireStructureLayout(context, item.id);
    const parent = item.parentId === null ? IDENTITY_STRUCTURE_TRANSFORM : transforms.get(item.parentId)!;
    const world = composeStructureTransforms(parent, structureLayoutTransform(layout));
    transforms.set(item.id, world); paintOrder.set(item.id, order);
    if (layout.kind !== "group") bounds.set(item.id, transformedStructureBounds(world, layout.size));
  }
  for (const item of ordered.toReversed()) {
    if (item.kind !== "group") continue;
    bounds.set(item.id, unionStructureBounds((context.tree.children.get(item.id) ?? []).map((child) => bounds.get(child.id)!))!);
  }
  return { transforms, bounds, paintOrder };
}
/** Tight bounds in the requested parent's coordinate system, not a re-rotated world AABB. */
export function structureBoundsInParent(context: StructureContext, projection: StructureProjection, ids: readonly string[], parentId: string | null): StructureBounds | undefined {
  const inverse = invertStructureTransform(parentId === null ? IDENTITY_STRUCTURE_TRANSFORM : projection.transforms.get(requireStructureItem(context.tree, parentId).id)!);
  const pending = [...ids]; const seen = new Set<string>(); const bounds: StructureBounds[] = [];
  while (pending.length) {
    const id = pending.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    const layout = requireStructureLayout(context, id);
    if (layout.kind === "group") {
      for (const child of context.tree.children.get(id) ?? []) pending.push(child.id);
    } else bounds.push(transformedStructureBounds(composeStructureTransforms(inverse, projection.transforms.get(id)!), layout.size));
  }
  return unionStructureBounds(bounds);
}
''',
"organization/structure/structure-snapshot-validation.ts": '''import { WORLD_COORDINATE_LIMIT, WORLD_SCALE_MAX, WORLD_SCALE_MIN } from "../../canvas/contract/structure-layout";
import { StructureError } from "../../model/structure/structure-error";
import { validateContentPlacements, validateStructure } from "../../model/structure/structure-validation";
import { indexStructureSnapshot } from "./structure-context";
import { projectStructure } from "./structure-projection";
import { structureSnapshotSchema, type StructureSnapshot } from "./structure-snapshot-contract";

export function parseStructureSnapshot(value: unknown, contentIds: ReadonlySet<string>): StructureSnapshot {
  const snapshot = structureSnapshotSchema.parse(value);
  const context = validateStructureSnapshot(snapshot);
  validateContentPlacements(context.tree, contentIds);
  return snapshot;
}
export function validateStructureSnapshot(snapshot: StructureSnapshot) {
  const context = indexStructureSnapshot(snapshot);
  validateStructure(context.tree);
  const projection = projectStructure(context);
  for (const [id, transform] of projection.transforms) {
    if (!validCoordinate(transform.x) || !validCoordinate(transform.y) || transform.scale < WORLD_SCALE_MIN || transform.scale > WORLD_SCALE_MAX) {
      throw new StructureError("invalid-layout", "Composed world transforms must be bounded and invertible", [id]);
    }
  }
  for (const [id, bounds] of projection.bounds) {
    if (bounds.width <= 0 || bounds.height <= 0 || ![bounds.x, bounds.y, bounds.x + bounds.width, bounds.y + bounds.height].every(validCoordinate)) {
      throw new StructureError("invalid-layout", "World bounds must remain finite and non-degenerate", [id]);
    }
  }
  return context;
}
function validCoordinate(value: number) { return Number.isFinite(value) && Math.abs(value) <= WORLD_COORDINATE_LIMIT; }
''',
"organization/structure/structure-selection.ts": '''import { StructureError } from "../../model/structure/structure-error";
import { requireStructureItem, structureAncestors, structureSubtree, type StructureIndex } from "../../model/structure/structure-index";

export function structureOperationRoots(index: StructureIndex, ids: readonly string[]) {
  const selected = new Set(ids);
  return [...selected].map((id) => requireStructureItem(index, id)).filter((item) => !structureAncestors(index, item.id).some((parent) => selected.has(parent.id)));
}
export function requireStructureRoots(index: StructureIndex, ids: readonly string[]) {
  const roots = structureOperationRoots(index, ids);
  if (!roots.length) throw new StructureError("invalid-selection", "The operation requires a selection");
  return roots;
}
export function assertStructureEditable(index: StructureIndex, ids: readonly string[]) {
  const blocked = ids.filter((id) => requireStructureItem(index, id).locked || structureAncestors(index, id).some((parent) => parent.locked));
  if (blocked.length) throw new StructureError("locked", "The operation is blocked by locked items or ancestors", blocked);
}
export function structureContentIds(index: StructureIndex, ids: readonly string[]) {
  return structureSubtree(index, structureOperationRoots(index, ids).map((item) => item.id)).flatMap((item) => item.kind === "content" ? [item.nodeId] : []);
}
export function containingStructureSection(index: StructureIndex, id: string) {
  const item = requireStructureItem(index, id);
  return item.kind === "section" ? item.id : structureAncestors(index, id).find((parent) => parent.kind === "section")?.id;
}
export function structureSelectionTarget(index: StructureIndex, id: string, scope: string | null): string | undefined {
  const item = requireStructureItem(index, id);
  const ancestors = structureAncestors(index, id);
  if (scope !== null) {
    if (requireStructureItem(index, scope).kind !== "group") throw new StructureError("invalid-selection", "Only Groups define an editing scope", [scope]);
    if (id === scope) return id;
    const scopeIndex = ancestors.findIndex((parent) => parent.id === scope);
    if (scopeIndex < 0) return undefined;
    ancestors.length = scopeIndex;
  }
  return ancestors.findLast((parent) => parent.kind === "group")?.id ?? item.id;
}
''',
"__tests__/structure-contract.test.ts": '''import { describe, expect, it } from "vitest";
import { parseStructureSnapshot } from "../organization/structure/structure-snapshot-validation";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../organization/structure/structure-projection";
import { structureContentIds, structureOperationRoots, structureSelectionTarget } from "../organization/structure/structure-selection";

const transform = { x: 0, y: 0, rotation: 0, scale: 1 };
function fixture() {
  return { structure: { version: 1, items: [
    { kind: "section", id: "s", parentId: null, order: 0, locked: false, title: "Sources" },
    { kind: "group", id: "g", parentId: "s", order: 0, locked: false, title: "Pair" },
    { kind: "content", id: "a", nodeId: "a", parentId: "g", order: 0, locked: false },
    { kind: "content", id: "b", nodeId: "b", parentId: "g", order: 1, locked: true },
  ] }, layout: { version: 1, items: [
    { kind: "section", itemId: "s", position: { x: 100, y: 200 }, size: { width: 400, height: 300 }, collapsed: false },
    { kind: "group", itemId: "g", transform: { ...transform, rotation: 45, scale: 2 } },
    { kind: "content", itemId: "a", transform, size: { width: 100, height: 80 }, sizeAuthority: "frame" },
    { kind: "content", itemId: "b", transform: { ...transform, x: 120 }, size: { width: 100, height: 80 }, sizeAuthority: "frame" },
  ] } };
}
const parse = (value: unknown) => parseStructureSnapshot(value, new Set(["a", "b"]));
describe("Space structure contract", () => {
  it("keeps structure and local layout separate and resolves operation roots", () => {
    const snapshot = parse(fixture());
    const context = indexStructureSnapshot(snapshot);
    expect(structureOperationRoots(context.tree, ["g", "a", "b"]).map((item) => item.id)).toEqual(["g"]);
    expect(structureContentIds(context.tree, ["s", "a"])).toEqual(["a", "b"]);
    expect(structureSelectionTarget(context.tree, "a", null)).toBe("g");
    expect(structureSelectionTarget(context.tree, "a", "g")).toBe("a");
  });
  it("derives tight local Group bounds without rotating world AABBs again", () => {
    const context = indexStructureSnapshot(parse(fixture()));
    const bounds = structureBoundsInParent(context, projectStructure(context), ["g"], "g")!;
    expect(bounds.x).toBeCloseTo(0); expect(bounds.y).toBeCloseTo(0);
    expect(bounds.width).toBeCloseTo(220); expect(bounds.height).toBeCloseTo(80);
  });
  it("rejects a missing active content placement", () => {
    expect(() => parseStructureSnapshot(fixture(), new Set(["a", "b", "missing"]))).toThrow(/placement/u);
  });
  it("rejects nested Sections", () => {
    const source = fixture(); source.structure.items[0]!.parentId = "g";
    expect(() => parse(source)).toThrow(/Space root/u);
  });
  it("rejects duplicate sibling orders", () => {
    const source = fixture(); source.structure.items[3]!.order = 0;
    expect(() => parse(source)).toThrow(/Sibling order/u);
  });
  it("rejects parent cycles without recursive traversal", () => {
    const source = fixture(); source.structure.items[1]!.parentId = "g";
    expect(() => parse(source)).toThrow(/cycle/u);
  });
  it("rejects kind-mismatched layouts", () => {
    const source = fixture(); source.layout.items[1]!.itemId = "a";
    expect(() => parse(source)).toThrow(/Layout kind/u);
  });
  it("rejects composed scales outside the contract even when locals are legal", () => {
    const source = fixture(); source.layout.items[1]!.transform = { ...transform, scale: 1_000_000 };
    expect(() => parse(source)).toThrow(/bounds/u);
  });
});
''',
}

# Reuse the existing neutral appearance contract; do not create a second definition.
path = ROOT / "canvas/contract/canvas-document.ts"
text = path.read_text()
text = text.replace("const canvasNodeAppearanceSchema = z", "export const canvasNodeAppearanceSchema = z")
text = text.replace("export export const canvasNodeAppearanceSchema", "export const canvasNodeAppearanceSchema")
path.write_text(text)
for name, content in FILES.items():
    file = ROOT / name
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(content)
subprocess.run(["pnpm", "exec", "oxfmt", str(path), *[str(ROOT / name) for name in FILES]], check=True)
