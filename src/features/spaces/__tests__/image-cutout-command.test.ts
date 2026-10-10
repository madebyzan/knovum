import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { createImageCutout } from "../knowledge-derivations/image-cutout";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
const calls = vi.hoisted(() => ({ start: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("../knowledge-derivations/start-action", () => ({ startDerivationAction: calls.start }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
const source: SpaceCanvasNode<SpaceImagePayload> = {
  id: "source",
  type: "image",
  initialPosition: { x: 10, y: 20 },
  data: {
    kind: "image",
    title: "Photo",
    fileName: "photo.png",
    assetPath: "/assets/photo.png",
    sourceRevision: "sha256:asset",
    knowledgeRevision: "node1:frozen",
  },
};
const execution = { type: "local", model: "u2netp", specId: "u2netp-spec" } as const;
beforeEach(() => {
  calls.start.mockReset().mockResolvedValue({ runId: "run" });
  mockIPC((command) => {
    if (command !== "plugin:spaces|get_image_operation_capabilities")
      throw new Error(`Unexpected command: ${command}`);
    return {
      targets: [
        { execution, capability: "imageSegmentation", availability: { state: "ready" }, selected: true },
      ],
    };
  });
});
afterEach(clearMocks);
it("submits one durable cutout invocation pinned to the selected model and source", async () => {
  const store = { activeSpaceId: "space" };
  await createImageCutout(store, source);
  expect(calls.start).toHaveBeenCalledOnce();
  expect(calls.start).toHaveBeenCalledWith(
    expect.objectContaining({
      store,
      actionId: "remove-image-background",
      sources: [{ nodeId: source.id, title: "Photo", role: "primary", sourceRevision: "node1:frozen" }],
      parameters: expect.objectContaining({ operation: { kind: "remove-background" }, execution }),
    }),
  );
});
it("does not select another model when the chosen model is unavailable", async () => {
  mockIPC(() => ({
    targets: [
      {
        execution,
        capability: "imageSegmentation",
        availability: { state: "blocked", reason: "configuration_incomplete" },
        selected: true,
      },
      {
        execution: { ...execution, model: "u2net" },
        capability: "imageSegmentation",
        availability: { state: "ready" },
        selected: false,
      },
    ],
  }));
  await expect(createImageCutout({ activeSpaceId: "space" }, source)).rejects.toThrow(
    "spaces.media.imageEditor.selection.modelUnavailable",
  );
  expect(calls.start).not.toHaveBeenCalled();
});
it("does not create a UI output when durable action acceptance fails", async () => {
  calls.start.mockResolvedValue(undefined);
  expect(await createImageCutout({ activeSpaceId: "space" }, source)).toBeUndefined();
  expect(calls.start).toHaveBeenCalledOnce();
});
