import { beforeEach, describe, expect, it, vi } from "vitest";

import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import type { ProvenanceSource } from "../../knowledge/model/provenance";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { CanvasPoint } from "../contracts/canvas-editor-port";
import { addPassageNode, appendPassage, besideNodes, withPassage } from "../passage-capture";
import type { CanvasNodeLayoutV1 } from "../schema/document";

const mocks = vi.hoisted(() => ({
  current: undefined as unknown,
  layouts: [] as unknown[],
  steps: [] as string[],
}));
const attachProvenance = vi.hoisted(() =>
  vi.fn<(target: { targetId: string }, source: ProvenanceSource) => Promise<void>>(async (target, source) => {
    mocks.steps.push(`kept ${source.nodeId} on ${target.targetId}`);
  }),
);
vi.mock("../../knowledge/provenance-actions", () => ({ attachProvenance }));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => mocks.current }));
vi.mock("../editor-session", () => ({
  currentCanvasEditor: () => ({ getDocument: () => ({ nodes: mocks.layouts }) }),
}));

function layout(nodeId: string, x: number, y: number, width = 200): CanvasNodeLayoutV1 {
  return { nodeId, frame: { x, y, width, height: 100, rotation: 0 }, sizingMode: "frame", zIndex: 0 };
}

function canvasNode(id: string, data: SpaceNodePayload, hidden = false) {
  return { id, type: data.kind, data, initialPosition: { x: 0, y: 0 }, hidden } as SpaceCanvasNode;
}

function store(nodes: SpaceCanvasNode[]) {
  const steps = mocks.steps;
  const current = {
    activeSpaceId: "space",
    nodes,
    addContent: vi.fn<(data: SpaceNodePayload, position?: CanvasPoint) => Promise<SpaceCanvasNode[]>>(
      async (data) => {
        steps.push("add");
        return [canvasNode("added", data)];
      },
    ),
    updateNodeContents: vi.fn<() => Promise<void>>(async () => void steps.push("update")),
    createRelation: vi.fn<(sourceId: string, targetId: string) => Promise<void>>(
      async (sourceId: string, targetId: string) => void steps.push(`${sourceId}->${targetId}`),
    ),
    recordHistoryMutation: vi.fn<(action: () => Promise<void>) => Promise<void>>(
      async (action: () => Promise<void>) => {
        steps.push("begin");
        await action();
        steps.push("end");
      },
    ),
  };
  mocks.current = current;
  return { current, steps };
}

const note = (text: string) => createSpaceNotePayload("Note", text);

function cited(nodeId: string, quote = `from ${nodeId}`): ProvenanceSource {
  return { nodeId, quote, locator: { sourceRevision: `${nodeId}-r1`, location: { kind: "node" } } };
}

beforeEach(() => {
  mocks.layouts = [];
  mocks.steps = [];
  attachProvenance.mockClear();
});

describe("where a passage goes", () => {
  it("stands right of the items it came from, top-aligned", () => {
    const layouts = [layout("a", 0, 40), layout("b", 100, 0, 300), layout("far", 2000, 0)];
    expect(besideNodes(layouts, ["a", "b"])).toEqual({ x: 448, y: 0 });
    expect(besideNodes(layouts, ["missing"])).toBeUndefined();
  });

  it("steps below passages already kept beside the same items", () => {
    const layouts = [layout("a", 0, 0), layout("kept", 248, 0), layout("kept-too", 248, 32)];
    expect(besideNodes(layouts, ["a"])).toEqual({ x: 248, y: 64 });
  });
});

describe("adding a passage to an item", () => {
  it("adds paragraphs to a note below a blank line, as its editor would save them", () => {
    const before = note("First\n\n");
    const after = withPassage(before, "Second\nThird");
    expect(after).toMatchObject({
      plainText: "First\n\nSecond\nThird",
      tiptapContent: {
        type: "doc",
        content: [
          { type: "paragraph", content: [{ type: "text", text: "First" }] },
          { type: "paragraph" },
          { type: "paragraph", content: [{ type: "text", text: "Second" }] },
          { type: "paragraph", content: [{ type: "text", text: "Third" }] },
        ],
      },
    });
    expect(withPassage(note(""), "Only")).toMatchObject({ plainText: "Only" });
  });

  it("adds a paragraph to Markdown and refuses every other kind", () => {
    expect(withPassage({ kind: "markdown", title: "M", text: "# Plan\n" }, "More")).toEqual({
      kind: "markdown",
      title: "M",
      text: "# Plan\n\nMore",
    });
    expect(
      withPassage({ kind: "website", title: "W", url: "https://a.b" } as SpaceNodePayload, "x"),
    ).toBeUndefined();
  });
});

describe("keeping a passage on the canvas", () => {
  it("places it beside its live sources, links them as one undoable step and keeps what it cites", async () => {
    mocks.layouts = [layout("source", 0, 0)];
    const { current, steps } = store([canvasNode("source", note("s")), canvasNode("gone", note("g"), true)]);
    await addPassageNode(note("Kept"), [cited("source"), cited("gone"), cited("source", "again")]);
    expect(current.addContent).toHaveBeenCalledWith(note("Kept"), { x: 248, y: 0 });
    expect(current.createRelation).toHaveBeenCalledExactlyOnceWith("added", "source", "derives-from", {
      spaceId: "space",
    });
    expect(steps).toEqual(["begin", "add", "added->source", "end", "kept source on added"]);
    expect(attachProvenance).toHaveBeenCalledExactlyOnceWith(
      { spaceId: "space", targetKind: "node", targetId: "added" },
      cited("source"),
    );
  });

  it("uses the drop point, and the default spot when nothing is cited", async () => {
    const { current } = store([]);
    await addPassageNode(note("Dropped"), [], { x: 5, y: 6 });
    await addPassageNode(note("Plain"), []);
    expect(current.addContent.mock.calls.map((call) => call[1])).toEqual([{ x: 5, y: 6 }, undefined]);
    expect(current.createRelation).not.toHaveBeenCalled();
    expect(attachProvenance).not.toHaveBeenCalled();
  });

  it("appends to a note and links it to the sources other than itself", async () => {
    const target = canvasNode("target", note("Before"));
    const { current, steps } = store([target, canvasNode("source", note("s"))]);
    await appendPassage("target", "After", [cited("target"), cited("source")]);
    expect(current.updateNodeContents).toHaveBeenCalledWith([
      { id: "target", before: target.data, after: withPassage(target.data, "After") },
    ]);
    expect(steps).toEqual(["begin", "update", "target->source", "end", "kept source on target"]);
    await expect(appendPassage("missing", "x", [])).rejects.toThrow("cannot take a passage");
  });
});
