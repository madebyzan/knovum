import { describe, expect, it } from "vitest";
import { applyStructureChanges } from "../organization/structure/commands/changes";
import { planStructureCommand } from "../organization/structure/commands/plan";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { contentFixture, fixtureWorld, groupFixture, nestedStructureFixture, sectionFixture, structureFixture } from "../testing/structure-fixtures";

describe("complete structure editing plans", () => {
  it("groups non-adjacent siblings at the frontmost selected position", () => {
    const before = structureFixture([contentFixture("a"), contentFixture("b", { order: 1 }), contentFixture("c", { order: 2 })]);
    const plan = planStructureCommand(before, { kind: "group-selection", ids: ["c", "a"], id: "g", title: "  Pair  " });
    const tree = indexStructureSnapshot(plan.after).tree;
    expect(tree.children.get(null)?.map((item) => item.id)).toEqual(["b", "g"]);
    expect(tree.children.get("g")?.map((item) => item.id)).toEqual(["a", "c"]);
    expect(tree.items.get("g")).toMatchObject({ title: "Pair" });
    expect(applyStructureChanges(plan.after, plan.inverse)).toEqual(before);
  });
  it("preserves world geometry when ungrouping below rotated and scaled ancestors", () => {
    const before = nestedStructureFixture();
    const plan = planStructureCommand(before, { kind: "dissolve-container", id: "h" });
    for (const id of ["a", "b", "c"]) {
      const old = fixtureWorld(before, id);
      expect(fixtureWorld(plan.after, id)).toEqual({ x: expect.closeTo(old.x, 7), y: expect.closeTo(old.y, 7), rotation: expect.closeTo(old.rotation, 7), scale: expect.closeTo(old.scale, 7) });
    }
    expect(plan.effects.removedContentNodeIds).toEqual([]);
    expect(applyStructureChanges(plan.after, plan.inverse)).toEqual(before);
  });
  it("moves all selected Sections consistently and deduplicates descendants", () => {
    const before = structureFixture([sectionFixture("s1"), contentFixture("a", { parentId: "s1", x: 20 }), sectionFixture("s2", { order: 1, x: 500 }), contentFixture("b", { parentId: "s2", x: 40 })]);
    const plan = planStructureCommand(before, { kind: "move-items", ids: ["s2", "a", "s1", "b"], delta: { x: 40, y: -20 } });
    expect(plan.changes.layouts).toHaveLength(2);
    for (const id of ["s1", "s2", "a", "b"]) {
      const old = fixtureWorld(before, id);
      expect(fixtureWorld(plan.after, id)).toEqual({ ...old, x: old.x + 40, y: old.y - 20 });
    }
    expect(plan.changes.items).toEqual([]);
  });
  it("resizes the north/west Section boundary without relocating or ejecting content", () => {
    const before = nestedStructureFixture();
    const plan = planStructureCommand(before, { kind: "resize-section", id: "s", bounds: { x: 200, y: 350, width: 60, height: 40 } });
    expect(plan.changes.items).toEqual([]);
    for (const id of ["g", "h", "a", "b", "c", "d"]) {
      const old = fixtureWorld(before, id);
      expect(fixtureWorld(plan.after, id)).toEqual({ ...old, x: expect.closeTo(old.x, 7), y: expect.closeTo(old.y, 7) });
    }
  });
  it("carries protected descendants but rejects deleting them", () => {
    const before = nestedStructureFixture();
    const moved = planStructureCommand(before, { kind: "move-items", ids: ["g", "c"], delta: { x: 30, y: 0 } });
    expect(fixtureWorld(moved.after, "c").x).toBeCloseTo(fixtureWorld(before, "c").x + 30);
    expect(() => planStructureCommand(before, { kind: "delete-subtrees", ids: ["g"] })).toThrow(/protected descendants/u);
  });
  it("commits drop displacement, parent change and source empty-group cleanup together", () => {
    const before = structureFixture([groupFixture("g", { x: 20 }), contentFixture("a", { parentId: "g", x: 10 }), sectionFixture("s", { order: 1, x: 500 })]);
    const plan = planStructureCommand(before, { kind: "drop-items", ids: ["a"], parentId: "s", delta: { x: 520, y: 60 } });
    expect(plan.after.structure.items.map((item) => item.id)).toEqual(["a", "s"]);
    expect(fixtureWorld(plan.after, "a")).toMatchObject({ x: 550, y: 60 });
    expect(indexStructureSnapshot(plan.after).tree.items.get("a")?.parentId).toBe("s");
    expect(applyStructureChanges(plan.after, plan.inverse)).toEqual(before);
  });
  it("does not reorder a drop inside the same parent", () => {
    const before = nestedStructureFixture();
    const plan = planStructureCommand(before, { kind: "drop-items", ids: ["a"], parentId: "h", delta: { x: 1, y: 1 } });
    expect(plan.changes.items).toEqual([]);
  });
  it("rejects cross-parent grouping and ancestor cycles without mutating input", () => {
    const before = nestedStructureFixture();
    const original = structuredClone(before);
    expect(() => planStructureCommand(before, { kind: "group-selection", ids: ["a", "d"], id: "new", title: "Invalid" })).toThrow(/same parent/u);
    expect(() => planStructureCommand(before, { kind: "reparent-items", ids: ["g"], parentId: "h" })).toThrow(/ancestor/u);
    expect(before).toEqual(original);
  });
  it("creates a drawn empty Section without inventing a member", () => {
    const plan = planStructureCommand(structureFixture(), { kind: "draw-section", id: "s", title: "Region", bounds: { x: 10, y: 20, width: 400, height: 300 }, ids: [] });
    expect(plan.after.structure.items).toHaveLength(1);
    expect(plan.after.layout.items[0]).toMatchObject({ position: { x: 10, y: 20 } });
  });
  it("wraps one Section's sibling content into a new root Section without moving content", () => {
    const before = structureFixture([sectionFixture("s", { x: 100 }), contentFixture("a", { parentId: "s", x: 50 }), contentFixture("b", { parentId: "s", order: 1, x: 250 })]);
    const plan = planStructureCommand(before, { kind: "section-around-selection", id: "new", title: "New", ids: ["a", "b"] });
    const tree = indexStructureSnapshot(plan.after).tree;
    expect(tree.items.get("new")?.parentId).toBeNull();
    expect(tree.items.has("s")).toBe(true);
    expect(tree.children.get("s")).toBeUndefined();
    expect(fixtureWorld(plan.after, "a")).toEqual(fixtureWorld(before, "a"));
    expect(applyStructureChanges(plan.after, plan.inverse)).toEqual(before);
  });
  it("replays owned exact records and rejects stale history", () => {
    const before = nestedStructureFixture();
    const plan = planStructureCommand(before, { kind: "move-items", ids: ["h"], delta: { x: 30, y: 40 } });
    const inverse = structuredClone(plan.inverse);
    const original = before.layout.items.find((item) => item.itemId === "h")!;
    if (original.kind !== "group") throw new Error("Fixture Group missing");
    original.transform.x = 999;
    expect(plan.inverse).toEqual(inverse);
    const later = planStructureCommand(plan.after, { kind: "move-items", ids: ["h"], delta: { x: 5, y: 0 } });
    expect(() => applyStructureChanges(later.after, plan.inverse)).toThrow(/changed after/u);
  });
});
