import { expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { canSplitImageLayers, splitImageLayers } from "../image-layers";

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

it("submits one durable split pinned to the source revision and requests no layer count", async () => {
  calls.start.mockReset().mockResolvedValue({ runId: "run" });
  await splitImageLayers(source());
  expect(calls.start).toHaveBeenCalledOnce();
  expect(calls.start).toHaveBeenCalledWith({
    actionId: "split-image-layers",
    sources: [
      { nodeId: "source", title: "Photo", sourceRevision: "node1:frozen", revisionBasis: "knowledge" },
    ],
    parameters: {},
    title: expect.stringContaining("spaces.creation.image.splitLayers"),
  });
});

it("does not split an image whose asset is still being prepared", async () => {
  calls.start.mockReset();
  const unpublished = [
    { sourceRevision: undefined },
    { knowledgeRevision: undefined },
    { assetPath: "" },
  ] satisfies Partial<SpaceImagePayload>[];
  for (const data of unpublished) expect(canSplitImageLayers(source(data))).toBe(false);
  const outcomes = await Promise.all(unpublished.map((data) => splitImageLayers(source(data))));
  expect(outcomes).toEqual(unpublished.map(() => undefined));
  expect(calls.start).not.toHaveBeenCalled();
});

it("reports a split that the Action layer refused", async () => {
  calls.start.mockReset().mockResolvedValue(undefined);
  expect(await splitImageLayers(source())).toBeUndefined();
  expect(calls.start).toHaveBeenCalledOnce();
});
