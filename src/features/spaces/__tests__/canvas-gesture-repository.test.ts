import { beforeEach, describe, expect, it, vi } from "vitest";

import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import { commitCanvasGesture } from "../canvas/persistence/canvas-gesture-repository";

const tauri = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: tauri.invoke }));

const document: CanvasDocumentV1 = {
  version: 1,
  viewport: { x: 0, y: 0, zoom: 1 },
  nodes: [],
};

describe("canvas gesture repository", () => {
  beforeEach(() => vi.clearAllMocks());

  it("constructs the strict persistence DTO without leaking gesture-only fields", async () => {
    tauri.invoke.mockResolvedValue({ revision: 4 });
    const mutation = { baseRevision: 3, document, assignments: [{ id: "node", sectionId: "section" }] };

    await commitCanvasGesture(mutation, "space", []);

    expect(tauri.invoke).toHaveBeenCalledWith("plugin:spaces|commit_canvas_gesture", {
      request: { spaceId: "space", baseRevision: 3, document, changes: [] },
    });
  });
});
