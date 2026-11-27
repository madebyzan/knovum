import { describe, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../../nodes/node-operations";
import { createEditingState } from "../editing-state";
import { persistSpaceTiptapContent } from "../tiptap-persistence";

function createEditorStub(markdown: string, plainText = markdown) {
  return {
    getMarkdown: () => markdown,
    getJSON: () => ({
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: plainText }] }],
    }),
    state: {
      doc: {
        content: { size: plainText.length },
        textBetween: () => plainText,
      },
    },
  };
}

describe("persistSpaceTiptapContent", () => {
  it("preserves an intentionally cleared markdown title without restoring from leading H1", async () => {
    const node: SpaceCanvasNode = {
      id: "doc-1",
      type: "markdown",
      initialPosition: { x: 0, y: 0 },
      data: {
        kind: "markdown",
        title: "中国股市研究",
        text: "# 中国股市研究\n\n正文内容",
      },
    };

    const state = createEditingState();
    state.title.value = "";
    const save = vi.fn<(change: NodePayloadChangeInput) => Promise<SpaceNodePayload>>(
      async (change) => change.data,
    );
    const onSaved = vi.fn<(payload: SpaceNodePayload) => void>();

    await persistSpaceTiptapContent({
      node,
      state,
      editor: createEditorStub("# 中国股市研究\n\n&nbsp;\n\n正文内容"),
      save,
      onSaved,
    });

    expect(save).toHaveBeenCalledOnce();
    const saved = onSaved.mock.calls[0]?.[0];
    expect(saved).toMatchObject({
      kind: "markdown",
      title: "",
      text: "# 中国股市研究\n\n正文内容",
    });
  });

  it("preserves an intentionally cleared note title without restoring from first line", async () => {
    const node: SpaceCanvasNode = {
      id: "note-1",
      type: "note",
      initialPosition: { x: 0, y: 0 },
      data: {
        kind: "note",
        title: "Old Note",
        plainText: "First line body",
        tiptapContent: { type: "doc", content: [{ type: "paragraph" }] },
      },
    };

    const state = createEditingState();
    state.title.value = "   ";
    const save = vi.fn<(change: NodePayloadChangeInput) => Promise<SpaceNodePayload>>(
      async (change) => change.data,
    );
    const onSaved = vi.fn<(payload: SpaceNodePayload) => void>();

    await persistSpaceTiptapContent({
      node,
      state,
      editor: createEditorStub("First line body"),
      save,
      onSaved,
    });

    expect(save).toHaveBeenCalledOnce();
    expect(onSaved).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "note",
        title: "",
      }),
    );
  });
});
