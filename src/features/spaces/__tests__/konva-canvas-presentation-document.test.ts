import { describe, expect, it } from "vitest";

import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { createCanvasSelectionSnapshot } from "../canvas/document/canvas-selection-snapshot";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { reorderCanvasNodes } from "../canvas/konva/document/canvas-layer-order";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("Konva canvas presentation document", () => {
  it("reorders selected nodes using the document z-order", () => {
    const nodes = createCanvasDocument([node("a", 0), node("b", 90), node("c", 180)]).nodes;
    expect(reorderCanvasNodes(nodes, new Set(["a"]), "front").map(({ nodeId }) => nodeId)).toEqual([
      "b",
      "c",
      "a",
    ]);
    expect(reorderCanvasNodes(nodes, new Set(["c"]), "back").map(({ nodeId }) => nodeId)).toEqual([
      "c",
      "a",
      "b",
    ]);
    expect(reorderCanvasNodes(nodes, new Set(["b"]), "forward").map(({ nodeId }) => nodeId)).toEqual([
      "a",
      "c",
      "b",
    ]);
  });

  it("undoes one relation color without replacing unrelated presentation changes", async () => {
    let historyEntry: { undo(): Promise<void>; redo(): Promise<void> } | undefined;
    const model = canvasModel([], (entry) => (historyEntry = entry));
    model.setRelationColor("a", "blue");
    const firstEntry = historyEntry;
    model.setRelationColor("b", "green");
    await firstEntry?.undo();
    expect(model.document.relationStyles).toEqual([{ relationId: "b", color: "green" }]);
  });

  it("records node appearance as a reversible document mutation", async () => {
    let historyEntry: { undo(): Promise<void>; redo(): Promise<void> } | undefined;
    const model = canvasModel([node("note", 0)], (entry) => (historyEntry = entry));
    model.setNodeAppearance("note", { fill: "green", fontWeight: "bold" });
    model.setNodeAppearance("note", { fontStyle: "italic" });
    await historyEntry?.undo();
    expect(model.document.nodes[0]?.appearance).toEqual({ fill: "green", fontWeight: "bold" });
    await historyEntry?.redo();
    expect(model.document.nodes[0]?.appearance).toEqual({
      fill: "green",
      fontStyle: "italic",
      fontWeight: "bold",
    });
  });

  it("projects selected appearance without a document-level subscription", () => {
    const layouts = createCanvasDocument([node("note", 0)]).nodes;
    layouts[0]!.appearance = { fill: "green", fontWeight: "bold" };
    expect(
      createCanvasSelectionSnapshot(layouts, new Set(["note"]), new Set(), new Map()).singleNodeAppearance,
    ).toEqual({ fill: "green", fontWeight: "bold" });
  });
});

function canvasModel(
  nodes: SpaceCanvasNode[],
  historyEntry: (entry: { undo(): Promise<void>; redo(): Promise<void> }) => void,
) {
  return new CanvasDocumentModel(createCanvasDocument(nodes), {
    documentChanged() {},
    historyEntry,
    viewportChanged() {},
  });
}

function node(id: string, x: number): SpaceCanvasNode {
  return { id, type: "note", data: createSpaceNotePayload(id, ""), initialPosition: { x, y: x } };
}
