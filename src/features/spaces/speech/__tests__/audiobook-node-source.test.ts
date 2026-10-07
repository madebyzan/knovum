import { beforeEach, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { readNodeAudiobookScript } from "../audiobook-script-import";

const invoke = vi.hoisted(() => vi.fn<(command: string, arguments_?: unknown) => Promise<unknown>>());
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
beforeEach(() => {
  invoke.mockReset();
});
function document(extension: string): SpaceCanvasNode {
  return {
    id: "document",
    type: "file",
    initialPosition: { x: 0, y: 0 },
    data: {
      kind: "file",
      title: "Manuscript",
      assetPath: `/managed/book.${extension}`,
      fileName: `book.${extension}`,
    },
  };
}

it("extracts the owning node and discards completion after the editor closes", async () => {
  let complete!: (value: unknown) => void;
  invoke.mockImplementation(
    () =>
      new Promise((resolve) => {
        complete = resolve;
      }),
  );
  let current = true;
  const pending = readNodeAudiobookScript(document("docx"), "Narrator", "job", () => current);
  expect(invoke).toHaveBeenCalledWith("plugin:spaces|read_audiobook_source", {
    sourceNodeId: "document",
    jobId: "job",
  });
  current = false;
  complete({ kind: "text", text: "Late response" });
  expect(await pending).toBeUndefined();
});

it("turns node document text into owned chapters and paragraphs", async () => {
  invoke.mockResolvedValue({ kind: "text", text: "# Chapter one\n\nFirst paragraph.\n\nSecond paragraph." });
  const script = await readNodeAudiobookScript(document("pdf"), "Narrator", "job", () => true);
  expect(script?.chapters).toEqual([
    {
      title: "Chapter one",
      paragraphs: [
        { role: "Narrator", text: "First paragraph." },
        { role: "Narrator", text: "Second paragraph." },
      ],
    },
  ]);
});

it("reads current text-node content without a file picker or native extraction", async () => {
  const node: SpaceCanvasNode = {
    id: "text",
    type: "markdown",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "markdown", title: "Current draft", text: "## Chapter\n\nCurrent words." },
  };
  const script = await readNodeAudiobookScript(node, "Narrator", "job", () => true);
  expect(script?.chapters[0]?.paragraphs[0]?.text).toBe("Current words.");
  expect(invoke).not.toHaveBeenCalled();
});

it("rejects structured node manuscripts with obsolete fields", async () => {
  invoke.mockResolvedValue({
    kind: "text",
    text: JSON.stringify({
      schemaVersion: 1,
      chapters: [{ title: "Chapter", paragraphs: [{ role: "Narrator", text: "Hello" }] }],
      legacy: true,
    }),
  });
  await expect(readNodeAudiobookScript(document("json"), "Narrator", "job", () => true)).rejects.toThrow(
    /Unrecognized key/u,
  );
});
