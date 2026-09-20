import { describe, expect, it } from "vitest";

import { createCanvasDocument, reconcileCanvasDocument } from "../canvas/document/canvas-document-sync";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { createCanvasPersistenceHistory } from "../canvas/persistence/canvas-persistence-history";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";

function node(id: string, x: number, y: number): SpaceCanvasNode {
  const data = createSpaceNotePayload(id, "");
  return { id, type: "note", data, initialPosition: { x, y } };
}

describe("Konva canvas document synchronization", () => {
  it("creates canonical frames from domain nodes", () => {
    const sectionData = { kind: "section" as const, title: "Research" };
    const section: SpaceCanvasNode = {
      id: "section",
      type: "section",
      data: sectionData,
      initialPosition: { x: 40, y: 60 },
    };
    const memberData = { ...createSpaceNotePayload("Note", "Body"), sectionId: "section", locked: true };
    const member: SpaceCanvasNode = {
      id: "member",
      type: "note",
      data: memberData,
      initialPosition: { x: 100, y: 140 },
    };

    const document = createCanvasDocument([section, member]);

    expect(document.nodes.map(({ nodeId }) => nodeId)).toEqual(["section", "member"]);
    expect(document.nodes[1]).toMatchObject({ nodeId: "member", zIndex: 1 });
    expect(document.nodes[1]).not.toHaveProperty("parentNodeId");
    expect(document.nodes[1]).not.toHaveProperty("locked");
  });

  it("adopts intrinsic media geometry until the persisted frame becomes authoritative", () => {
    const pending: SpaceCanvasNode = {
      id: "image",
      type: "image",
      data: imagePayload(),
      initialPosition: { x: 24, y: 32 },
    };
    const document = createCanvasDocument([pending]);
    const enriched = { ...pending, data: imagePayload(1600, 900) };

    const intrinsic = reconcileCanvasDocument(document, [enriched]);
    expect(intrinsic.nodes[0]).toMatchObject({
      sizeAuthority: "content",
      frame: { width: 420, height: 288 },
    });

    const authored = {
      ...intrinsic,
      nodes: [
        {
          ...intrinsic.nodes[0]!,
          frame: { ...intrinsic.nodes[0]!.frame, width: 600, height: 400 },
          sizeAuthority: "frame" as const,
        },
      ],
    };
    expect(reconcileCanvasDocument(authored, [enriched])).toBe(authored);
  });

  it("preserves authored frames while adding and removing domain nodes", () => {
    const document = createCanvasDocument([node("a", 0, 0), node("stale", 20, 20)]);
    document.nodes[0]!.frame.x = 720;

    const next = reconcileCanvasDocument(document, [node("a", 0, 0), node("b", 90, 120)]);

    expect(next.nodes.map(({ nodeId }) => nodeId)).toEqual(["a", "b"]);
    expect(next.nodes[0]!.frame.x).toBe(720);
    expect(next.nodes[1]!.frame).toMatchObject({ x: 90, y: 120 });
  });

  it("rejects content authority on nodes without content-sized geometry", () => {
    const note = node("note", 0, 0);
    const document = createCanvasDocument([note]);
    document.nodes[0]!.sizeAuthority = "content";

    expect(() => reconcileCanvasDocument(document, [note])).toThrow(
      "Node note does not support content-authoritative sizing",
    );
  });

  it("preserves document and layout identities when domain geometry is unchanged", () => {
    const nodes = [node("a", 0, 0), node("b", 90, 120)];
    const document = createCanvasDocument(nodes);

    expect(reconcileCanvasDocument(document, nodes)).toBe(document);
  });

  it("keeps viewport changes made after an import when undoing imported content", async () => {
    const before = createCanvasDocument([node("a", 0, 0)]);
    const model = new CanvasDocumentModel(before, {
      documentChanged() {},
      historyEntry() {},
      viewportChanged() {},
    });
    const after = {
      ...model.document,
      nodes: [
        ...model.document.nodes,
        {
          nodeId: "b",
          frame: { x: 10, y: 10, width: 100, height: 100, rotation: 0 },
          sizeAuthority: "frame" as const,
          zIndex: 1,
        },
      ],
    };
    model.applyLocalDocument(after);
    const entry = createCanvasPersistenceHistory({
      before,
      after,
      persistence: {
        undo: async () => ({ revision: 2 }),
        redo: async () => ({ revision: 3 }),
      },
      current: () => model.document,
      commit: async (document, transition) => ({
        ...(await transition({ baseRevision: 1, document })),
        document,
      }),
      adopt: (document) => model.applyLocalDocument(document),
    });
    model.setViewport({ x: 90, y: 120, zoom: 2 });

    await entry.undo();

    expect(model.document.viewport).toEqual({ x: 90, y: 120, zoom: 2 });
    expect(model.document.nodes.map(({ nodeId }) => nodeId)).toEqual(["a"]);
  });
});

function imagePayload(intrinsicWidth?: number, intrinsicHeight?: number): SpaceImagePayload {
  return {
    kind: "image",
    title: "Reference",
    assetPath: "assets/reference.png",
    fileName: "reference.png",
    ...(intrinsicWidth && intrinsicHeight ? { intrinsicWidth, intrinsicHeight } : {}),
  };
}
