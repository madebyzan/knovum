import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef, type EffectScope } from "vue";

const mocks = vi.hoisted(() => ({
  prepareTextEmbedding: vi.fn<(title: string) => Promise<boolean>>(),
  scheduleSpaceKnowledgeIndex: vi.fn<(spaceId: string) => void>(),
  toastError: vi.fn<(message: string) => void>(),
}));

vi.mock("@tanstack/vue-query", () => ({
  useQueryClient: () => ({ invalidateQueries: vi.fn<() => Promise<void>>() }),
  useQuery: () => ({}),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../indexing/prepare-text-embedding", () => ({
  prepareTextEmbedding: mocks.prepareTextEmbedding,
}));
vi.mock("../../indexing/index-scheduler", () => ({
  scheduleSpaceKnowledgeIndex: mocks.scheduleSpaceKnowledgeIndex,
  subscribeKnowledgeIndexStatus: () => () => undefined,
}));

import { useRelatedContent } from "../use-related-content";

let scope: EffectScope;

describe("related content resource preparation", () => {
  beforeEach(() => {
    scope = effectScope();
    vi.clearAllMocks();
    mocks.prepareTextEmbedding.mockResolvedValue(true);
  });
  afterEach(() => scope.stop());

  it("prepares embedding resources before scheduling a full Space index", async () => {
    let finish!: (prepared: boolean) => void;
    mocks.prepareTextEmbedding.mockReturnValue(new Promise((resolve) => (finish = resolve)));
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;

    const pending = related.prepare();
    expect(related.preparing.value).toBe(true);
    expect(mocks.prepareTextEmbedding).toHaveBeenCalledWith("spaces.search.semantic.preparing");
    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    await related.prepare();
    expect(mocks.prepareTextEmbedding).toHaveBeenCalledOnce();
    finish(true);
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

  it("stays quiet without scheduling an index when nothing was prepared", async () => {
    mocks.prepareTextEmbedding.mockResolvedValue(false);
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;
    await related.prepare();

    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    expect(mocks.toastError).not.toHaveBeenCalled();
    expect(related.preparing.value).toBe(false);
  });

  it("shows preparation failures and permits a later retry", async () => {
    mocks.prepareTextEmbedding.mockRejectedValueOnce(new Error("Download failed"));
    const related = scope.run(() => useRelatedContent("space-1", "node-1"))!;
    await related.prepare();

    expect(mocks.toastError).toHaveBeenCalledWith("Download failed");
    expect(mocks.scheduleSpaceKnowledgeIndex).not.toHaveBeenCalled();
    expect(related.preparing.value).toBe(false);
    await related.prepare();
    expect(mocks.scheduleSpaceKnowledgeIndex).toHaveBeenCalledExactlyOnceWith("space-1");
  });
});
