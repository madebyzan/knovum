import { describe, expect, it } from "vitest";
import { reactive } from "vue";

import { createSpaceNode } from "../canvas/node";
import { readSpaceClipboard, writeSpaceClipboard } from "../clipboard/transfer";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

function clipboardEvent() {
  const data = new Map<string, string>();
  return {
    event: {
      clipboardData: {
        getData: (type: string) => data.get(type) ?? "",
        setData: (type: string, value: string) => void data.set(type, value),
      },
    } as unknown as ClipboardEvent,
    data,
  };
}

describe("spaces clipboard", () => {
  it("round-trips the structured HTML marker while keeping useful plain text", () => {
    const node = createSpaceNode(createSpaceNotePayload("Idea", "Useful text"), { x: 12, y: 18 }, "note-1");
    const { event, data } = clipboardEvent();

    expect(writeSpaceClipboard(event, [{ node, position: { x: 12, y: 18 } }])).toBe(true);
    expect(data.get("text/plain")).toBe("Useful text");
    expect(readSpaceClipboard(event)?.nodes[0]).toEqual({
      id: "note-1",
      position: { x: 12, y: 18 },
      data: node.data,
    });
  });

  it("copies reactive node data without structured-clone failures", () => {
    const node = createSpaceNode(
      reactive<SpaceNodePayload>({ ...createSpaceNotePayload("Idea", "Body"), tags: ["work"] }),
      { x: 12, y: 18 },
      "note-1",
    );
    const { event } = clipboardEvent();

    expect(writeSpaceClipboard(event, [{ node, position: { x: 12, y: 18 } }])).toBe(true);
    expect(readSpaceClipboard(event)?.nodes[0]?.data).toEqual({
      ...createSpaceNotePayload("Idea", "Body"),
      tags: ["work"],
    });
  });

  it("rejects any clipboard payload that is not version 1", () => {
    const { event, data } = clipboardEvent();
    const oldPayload = encodeURIComponent(
      JSON.stringify({
        version: 2,
        nodes: [{ id: "old", position: { x: 0, y: 0 }, data: createSpaceNotePayload("Old") }],
      }),
    );
    data.set("text/html", `<!--knovum-spaces:${oldPayload}-->`);

    expect(readSpaceClipboard(event)).toBeUndefined();
  });
});
