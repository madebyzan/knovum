import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef, type EffectScope } from "vue";

const mocks = vi.hoisted(() => ({
  startResourcePreparation: vi.fn<() => { completion: Promise<void> }>(),
  scheduleSpaceKnowledgeIndex: vi.fn<(spaceId: string) => void>(),
  toastError: vi.fn<(message: string) => void>(),
}));

vi.mock("@tanstack/vue-query", () => ({
  useQueryClient: () => ({ invalidateQueries: vi.fn<() => Promise<void>>() }),
  useQuery: () => ({}),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../../foundation/runtime/resource-preparation", () => ({
  startResourcePreparation: mocks.startResourcePreparation,
}));
vi.mock("../knowledge/indexing/index-scheduler", () => ({
  scheduleSpaceKnowledgeIndex: mocks.scheduleSpaceKnowledgeIndex,
  subscribeKnowledgeIndexStatus: () => () => undefined,
}));

import { useRelatedContent } from "../knowledge/queries/use-related-content";

let scope: EffectScope;

describe("related content resource preparation", () => {
  beforeEach(() => {
    scope = effectScope();
    vi.clearAllMocks();
    mocks.startResourcePreparation.mockReturnValue({ completion: Promise.resolve() });
  });
  afterEach(() => scope.stop());

  it("prepares embedding resources before scheduling a full Space index", async () => {
    let finish!: () => void;
    mocks.startResourcePreparation.mockReturnValue({
      completion: new Promise<void>((resolve) => (finish = resolve)),
    });
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;

    const pending = related.prepare();
    expect(related.preparing.value).toBe(true);
    expect(mocks.startResourcePreparation).toHaveBeenCalledWith(
      { kind: "textEmbedding" },
      "spaces.search.semantic.preparing",
    );
    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    await related.prepare();
    expect(mocks.startResourcePreparation).toHaveBeenCalledOnce();
    finish();
    await pending;

    expect(mocks.scheduleSpaceKnowledgeIndex).toHaveBeenCalledExactlyOnceWith("space-1");
    expect(related.preparing.value).toBe(false);
  });

  it("does not schedule an index when the active Space changes during preparation", async () => {
    const spaceId = shallowRef("space-1");
    const related = scope.run(() => useRelatedContent(spaceId, "node-1"))!;
    const pending = related.prepare();
    spaceId.value = "space-2";
    await pending;

    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    expect(related.preparing.value).toBe(false);
  });

  it("keeps cancellation quiet without scheduling an index", async () => {
    mocks.startResourcePreparation.mockReturnValue({
      completion: Promise.reject({ code: "local_resource_preparation_cancelled", message: "Cancelled" }),
    });
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;
    await related.prepare();

    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    expect(mocks.toastError).not.toHaveBeenCalled();
    expect(related.preparing.value).toBe(false);
  });

  it("shows preparation failures and permits a later retry", async () => {
    mocks.startResourcePreparation.mockReturnValueOnce({
      completion: Promise.reject(new Error("Download failed")),
    });
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;
    await related.prepare();

    expect(mocks.toastError).toHaveBeenCalledWith("Download failed");
    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    expect(related.preparing.value).toBe(false);
    await related.prepare();
    expect(mocks.scheduleSpaceKnowledgeIndex).toHaveBeenCalledExactlyOnceWith("space-1");
  });
});
