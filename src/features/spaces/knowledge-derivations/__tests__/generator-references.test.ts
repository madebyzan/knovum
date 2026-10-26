import { expect, it } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { pinnedReferences, readReference } from "../generators/generator-references";

function node(id: string, data: Record<string, unknown>) {
  return { id, data: { title: id, ...data } } as unknown as SpaceCanvasNode;
}

const photo = node("photo", { kind: "image", sourceRevision: "asset:1" });
const arriving = node("arriving", { kind: "image", processingState: "importing" });
const broken = node("broken", { kind: "image", processingState: "failed" });
const emptyNote = node("note", { kind: "note", plainText: "  " });

it("pins what can be read, waits for what is still arriving, and refuses what never will be", () => {
  expect(readReference(photo, false)).toEqual({
    state: "ready",
    source: { nodeId: "photo", title: "photo", sourceRevision: "asset:1", revisionBasis: "asset" },
  });
  expect(readReference(arriving, false)).toEqual({ state: "preparing" });
  expect(readReference(emptyNote, true)).toEqual({ state: "preparing" });
  expect(readReference(broken, false)).toEqual({ state: "unreadable" });
  expect(readReference(emptyNote, false)).toEqual({ state: "unreadable" });
});

it("pins every reference or none, so a Run never reads part of what was asked", () => {
  const ready = readReference(photo, false);
  expect(pinnedReferences([ready])).toHaveLength(1);
  expect(pinnedReferences([])).toEqual([]);
  expect(pinnedReferences([ready, readReference(arriving, false)])).toBeUndefined();
});
