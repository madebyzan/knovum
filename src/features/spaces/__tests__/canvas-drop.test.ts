import { beforeEach, describe, expect, it, vi } from "vitest";

import type { importSpaceBatch } from "../assets/import-batch";
import { importDroppedFiles } from "../canvas/drop";
import { createSpaceNode } from "../canvas/node";

const mocks = vi.hoisted(() => ({
  current: undefined as unknown,
  importSpaceBatch: vi.fn<typeof importSpaceBatch>(),
  spaceImportContext: vi.fn<() => object>(() => ({})),
}));

vi.mock("../session/space-store", () => ({ useSpaceStore: () => mocks.current }));
vi.mock("../assets/import-batch", () => ({ importSpaceBatch: mocks.importSpaceBatch }));
vi.mock("../assets/import-context", () => ({ spaceImportContext: mocks.spaceImportContext }));

function store() {
  return (mocks.current = {
    activeSpaceId: "space-a",
    focusedEditorId: undefined,
    requestReveal: vi.fn<(ids: readonly string[]) => void>(),
  });
}

describe("canvas file drop", () => {
  beforeEach(() => vi.clearAllMocks());

  it("reveals successful imports and exposes failed imports", async () => {
    const current = store();
    const failure = new Error("Unreadable file");
    const imported = createSpaceNode(
      { kind: "note", title: "Imported", tiptapContent: { type: "doc" }, plainText: "" },
      { x: 80, y: 120 },
      "node-one",
    );
    mocks.importSpaceBatch.mockResolvedValue([
      { status: "imported", key: "/one.pdf", nodes: [imported] },
      { status: "failed", key: "/two.pdf", error: failure },
    ]);

    await expect(
      importDroppedFiles(
        { paths: ["/one.pdf", "/two.pdf"], position: { x: 0, y: 0 } },
        {
          x: 80,
          y: 120,
        },
      ),
    ).rejects.toBe(failure);
    expect(current.requestReveal).toHaveBeenCalledWith(["node-one"]);
  });

  it("treats user cancellation as a normal outcome", async () => {
    const current = store();
    mocks.importSpaceBatch.mockResolvedValue([{ status: "cancelled", key: "/one.pdf" }]);

    await expect(
      importDroppedFiles(
        { paths: ["/one.pdf"], position: { x: 0, y: 0 } },
        {
          x: 80,
          y: 120,
        },
      ),
    ).resolves.toBeUndefined();
    expect(current.requestReveal).not.toHaveBeenCalled();
  });
});
