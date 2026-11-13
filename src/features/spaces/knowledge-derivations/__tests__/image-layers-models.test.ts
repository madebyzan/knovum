import { beforeEach, describe, expect, it, vi } from "vitest";

import type { LocalModelAvailability } from "../../../../shared/contracts/generated/LocalModelAvailability";
import {
  imageTargetOption as target,
  localModelAvailability as model,
} from "../../__tests__/support/image-operation-fixtures";
import type { ImageOperationCapabilities } from "../../contracts/generated/ImageOperationCapabilities";
import type { ImageOperationTargetOption } from "../../contracts/generated/ImageOperationTargetOption";
import { readImageLayersModelChoice } from "../image-layers-models";

const mocks = vi.hoisted(() => ({
  capabilities: vi.fn<() => Promise<ImageOperationCapabilities>>(),
  fetchQuery: vi.fn<() => Promise<LocalModelAvailability[]>>(),
}));
vi.mock("../image-operation", () => ({ getImageOperationCapabilities: mocks.capabilities }));
vi.mock("../../../../foundation/query/query-client", () => ({
  queryClient: { fetchQuery: mocks.fetchQuery },
}));

function capabilities(targets: ImageOperationTargetOption[]): ImageOperationCapabilities {
  return {
    targets,
    remote: null,
    selection: { state: "ready" },
    depth: { state: "ready" },
    upscale: { state: "ready" },
    multiAngle: { state: "ready" },
    maskedRemoval: { state: "ready" },
    promptRemoval: { state: "ready" },
  };
}

const SINGLE_TARGETS = [
  target("birefnetLite", "imageSegmentation", { selected: true }),
  target("mobileSam", "imageSelection", { selected: true }),
  target("depthAnythingV2Small", "imageDepth", { selected: true }),
  target("lama", "imageInpainting", { selected: true }),
];
const SINGLE_MODELS = [
  model("birefnetLite", "imageSegmentation", { selected: true }),
  model("mobileSam", "imageSelection", { selected: true }),
  model("depthAnythingV2Small", "imageDepth", { selected: true }),
  model("lama", "imageInpainting", { selected: true }),
];

beforeEach(() => {
  mocks.capabilities.mockReset();
  mocks.fetchQuery.mockReset();
});

describe("readImageLayersModelChoice", () => {
  it("runs directly when every pass has one verified model", async () => {
    mocks.capabilities.mockResolvedValue(capabilities(SINGLE_TARGETS));
    mocks.fetchQuery.mockResolvedValue(SINGLE_MODELS);
    const decision = await readImageLayersModelChoice();
    expect(decision.kind).toBe("run");
    if (decision.kind !== "run") return;
    expect(decision.models.saliency).toEqual({
      type: "local",
      model: "birefnetLite",
      specId: "birefnetLite-spec",
    });
    expect(decision.models.inpainting).toEqual({ type: "local", model: "lama", specId: "lama-spec" });
  });

  it("opens a choice only for the passes with several verified models", async () => {
    mocks.capabilities.mockResolvedValue(
      capabilities([...SINGLE_TARGETS, target("ben2Base", "imageSegmentation")]),
    );
    mocks.fetchQuery.mockResolvedValue([...SINGLE_MODELS, model("ben2Base", "imageSegmentation")]);
    const decision = await readImageLayersModelChoice();
    expect(decision.kind).toBe("choose");
    if (decision.kind !== "choose") return;
    expect(decision.groups.map((group) => group.key)).toEqual(["saliency"]);
    expect(decision.resolved.selection).toEqual({
      type: "local",
      model: "mobileSam",
      specId: "mobileSam-spec",
    });
  });

  it("keeps the Settings preference for a pass with no verified download", async () => {
    mocks.capabilities.mockResolvedValue(capabilities(SINGLE_TARGETS));
    mocks.fetchQuery.mockResolvedValue(
      SINGLE_MODELS.map((entry) =>
        entry.id === "lama" ? Object.assign({}, entry, { verified: false, installed: false }) : entry,
      ),
    );
    const decision = await readImageLayersModelChoice();
    expect(decision.kind).toBe("run");
    if (decision.kind !== "run") return;
    expect(decision.models.inpainting).toEqual({ type: "local", model: "lama", specId: "lama-spec" });
  });
});
