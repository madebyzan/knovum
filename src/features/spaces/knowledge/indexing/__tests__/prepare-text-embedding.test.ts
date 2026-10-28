import { beforeEach, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  admit: vi.fn<(...args: unknown[]) => Promise<boolean>>(),
  start: vi.fn<(...args: unknown[]) => { completion: Promise<void> }>(),
}));
vi.mock("../../../../../foundation/ai/ai-readiness-admission", () => ({ admitAiRequirements: mocks.admit }));
vi.mock("../../../../../foundation/runtime/resource-preparation", async (importOriginal) => ({
  ...(await importOriginal<typeof ResourcePreparation>()),
  startResourcePreparation: mocks.start,
}));

import type * as ResourcePreparation from "../../../../../foundation/runtime/resource-preparation";
import { prepareTextEmbedding } from "../prepare-text-embedding";

beforeEach(() => {
  vi.clearAllMocks();
  mocks.admit.mockResolvedValue(true);
  mocks.start.mockReturnValue({ completion: Promise.resolve() });
});

it("admits the embedding model and prepares it as one titled task", async () => {
  await expect(prepareTextEmbedding("Preparing")).resolves.toBe(true);
  expect(mocks.admit).toHaveBeenCalledWith([{ kind: "localCapability", capability: "embedding" }]);
  expect(mocks.start).toHaveBeenCalledWith({ kind: "textEmbedding" }, "Preparing");
});

it("prepares nothing the admission refused", async () => {
  mocks.admit.mockResolvedValue(false);
  await expect(prepareTextEmbedding("Preparing")).resolves.toBe(false);
  expect(mocks.start).not.toHaveBeenCalled();
});

it("treats a cancelled download as nothing prepared rather than a failure", async () => {
  mocks.start.mockReturnValue({
    completion: Promise.reject({ code: "local_resource_preparation_cancelled", message: "Cancelled" }),
  });
  await expect(prepareTextEmbedding("Preparing")).resolves.toBe(false);
});

it("reports a failed download", async () => {
  mocks.start.mockReturnValue({ completion: Promise.reject(new Error("Download failed")) });
  await expect(prepareTextEmbedding("Preparing")).rejects.toThrow("Download failed");
});
