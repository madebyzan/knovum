import { describe, expect, it } from "vitest";

import { requireMovableScope, resolveCanvasMovementScope } from "../organization/movement-scope";
import { canvasNote } from "../testing/canvas-fixtures";
import { movementScene } from "../testing/canvas-movement-fixture";

const scene = movementScene;

describe("canvas operation scope", () => {
  it("distinguishes explicit selection, operation roots and affected descendants", () => {
    const scope = resolveCanvasMovementScope(scene(), new Set(["s1", "s2", "a", "b"]));
    expect([...scope.rootIds]).toEqual(["s1", "s2"]);
    expect(scope.affectedIds).toEqual(new Set(["s1", "s2", "a", "b"]));
    expect(scope.ownerById.get("a")).toBe("s1");
    expect(scope.ownerById.get("b")).toBe("s2");
  });
  it("does not adopt geometrically overlapping unassigned content", () => {
    const nodes = scene();
    nodes[2]!.data = canvasNote("a").data;
    const scope = resolveCanvasMovementScope(nodes, new Set(["s1"]));
    expect(scope.affectedIds).toEqual(new Set(["s1"]));
  });
  it("lets a locked child move with its unlocked container", () => {
    const nodes = scene();
    nodes[2]!.data = { ...nodes[2]!.data, locked: true };
    const scope = requireMovableScope(resolveCanvasMovementScope(nodes, new Set(["s1", "a"])));
    expect(scope.blockedIds).toEqual([]);
    expect(scope.affectedIds).toEqual(new Set(["s1", "a"]));
  });
  it("rejects the entire directly selected set when a root is locked", () => {
    const nodes = scene();
    nodes[3]!.data = { ...nodes[3]!.data, locked: true };
    const scope = resolveCanvasMovementScope(nodes, new Set(["s1", "b"]));
    expect(scope.blockedIds).toEqual(["b"]);
    expect(() => requireMovableScope(scope)).toThrow(/Unlock/u);
  });
  it("blocks direct editing inside a locked Section", () => {
    const nodes = scene();
    nodes[0]!.data = { ...nodes[0]!.data, locked: true };
    const scope = resolveCanvasMovementScope(nodes, new Set(["a"]));
    expect(scope.blockedIds).toEqual(["a"]);
    expect(() => requireMovableScope(scope)).toThrow(/Section/u);
  });
  it("fails on unknown requested identities instead of partly executing", () => {
    expect(() => resolveCanvasMovementScope(scene(), new Set(["s1", "missing"]))).toThrow(/unavailable/u);
  });
  it("leaves source records and selection immutable", () => {
    const nodes = scene();
    const before = structuredClone(nodes);
    const selection = new Set(["s1", "a"]);
    resolveCanvasMovementScope(nodes, selection);
    expect(nodes).toEqual(before);
    expect(selection).toEqual(new Set(["s1", "a"]));
  });
});
