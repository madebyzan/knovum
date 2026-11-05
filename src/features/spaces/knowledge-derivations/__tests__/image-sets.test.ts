import { expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { canDeriveImageSet, deriveImageSet } from "../image-sets";

const calls = vi.hoisted(() => ({ start: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("../start-action", () => ({ startDerivationAction: calls.start }));
vi.mock("../../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));

function source(data: Partial<SpaceImagePayload> = {}): SpaceCanvasNode<SpaceImagePayload> {
  return {
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
      ...data,
    },
  };
}

it.each([
  ["split-image-layers", "spaces.creation.image.splitLayers"],
  ["split-sticker-sheet", "spaces.creation.image.sliceStickers"],
] as const)(
  "submits one durable %s pinned to the source revision with no count",
  async (actionId, titleKey) => {
    calls.start.mockReset().mockResolvedValue({ runId: "run" });
    await deriveImageSet(actionId, source());
    expect(calls.start).toHaveBeenCalledOnce();
    expect(calls.start).toHaveBeenCalledWith({
      actionId,
      sources: [
        { nodeId: "source", title: "Photo", sourceRevision: "node1:frozen", revisionBasis: "knowledge" },
      ],
      parameters: {},
      title: expect.stringContaining(titleKey),
    });
  },
);

it("does not read an image whose asset is still being prepared", async () => {
  calls.start.mockReset();
  const unpublished = [
    { sourceRevision: undefined },
    { knowledgeRevision: undefined },
    { assetPath: "" },
  ] satisfies Partial<SpaceImagePayload>[];
  for (const data of unpublished) expect(canDeriveImageSet(source(data))).toBe(false);
  const outcomes = await Promise.all(
    unpublished.map((data) => deriveImageSet("split-sticker-sheet", source(data))),
  );
  expect(outcomes).toEqual(unpublished.map(() => undefined));
  expect(calls.start).not.toHaveBeenCalled();
});

it("reports a set that the Action layer refused", async () => {
  calls.start.mockReset().mockResolvedValue(undefined);
  expect(await deriveImageSet("split-image-layers", source())).toBeUndefined();
  expect(calls.start).toHaveBeenCalledOnce();
});
