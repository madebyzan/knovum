import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { LocalResourcesStatus } from "../../../../shared/contracts/generated/LocalResourcesStatus";

const mocks = vi.hoisted(() => ({
  getLocalResourcesStatus: vi.fn<() => Promise<LocalResourcesStatus>>(),
  textEmbeddingReady: vi.fn<(status?: LocalResourcesStatus) => boolean>(),
  indexKnowledgeRetrieval:
    vi.fn<
      (spaceId: string, nodeIds?: readonly string[]) => Promise<{ indexed: number; staleNodeIds: string[] }>
    >(),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../../../../foundation/runtime/local-resources", () => ({
  getLocalResourcesStatus: mocks.getLocalResourcesStatus,
  textEmbeddingReady: mocks.textEmbeddingReady,
}));
vi.mock("../persistence/retrieval-index", () => ({
  indexKnowledgeRetrieval: mocks.indexKnowledgeRetrieval,
}));

import {
  knowledgeIndexStatus,
  releaseKnowledgeIndex,
  scheduleKnowledgeIndex,
  scheduleSpaceKnowledgeIndex,
} from "../indexing/index-scheduler";

describe("spaces knowledge index scheduler", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    mocks.indexKnowledgeRetrieval.mockReset();
    mocks.indexKnowledgeRetrieval.mockResolvedValue({ indexed: 1, staleNodeIds: [] });
    mocks.getLocalResourcesStatus.mockReset();
    mocks.textEmbeddingReady.mockReset().mockReturnValue(true);
    mocks.logError.mockReset().mockResolvedValue(undefined);
  });

  afterEach(() => {
    releaseKnowledgeIndex("test-space");
    releaseKnowledgeIndex("structured-ai-error");
    vi.useRealTimers();
  });

  it("leaves startup indexing unprepared when embedding resources are unavailable", async () => {
    mocks.textEmbeddingReady.mockReturnValue(false);

    scheduleSpaceKnowledgeIndex("test-space");
    await vi.runAllTimersAsync();

    expect(mocks.getLocalResourcesStatus).toHaveBeenCalledOnce();
    expect(mocks.indexKnowledgeRetrieval).not.toHaveBeenCalled();
    expect(mocks.logError).not.toHaveBeenCalled();
    expect(knowledgeIndexStatus("test-space")).toBe("unprepared");
  });

  it("checks fresh readiness and indexes the entire Space after preparation", async () => {
    mocks.textEmbeddingReady.mockReturnValueOnce(false);
    scheduleSpaceKnowledgeIndex("test-space");
    await vi.runAllTimersAsync();

    scheduleSpaceKnowledgeIndex("test-space");
    await vi.runAllTimersAsync();

    expect(mocks.getLocalResourcesStatus).toHaveBeenCalledTimes(2);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledExactlyOnceWith("test-space", undefined);
    expect(knowledgeIndexStatus("test-space")).toBe("ready");
  });

  it("does not start indexing if the Space closes during the readiness check", async () => {
    let finish!: () => void;
    mocks.getLocalResourcesStatus.mockImplementationOnce(
      () =>
        new Promise<LocalResourcesStatus>((resolve) => (finish = () => resolve({} as LocalResourcesStatus))),
    );
    scheduleSpaceKnowledgeIndex("test-space");
    await vi.advanceTimersByTimeAsync(5_000);
    releaseKnowledgeIndex("test-space");
    finish();
    await vi.runAllTimersAsync();

    expect(mocks.indexKnowledgeRetrieval).not.toHaveBeenCalled();
    expect(knowledgeIndexStatus("test-space")).toBe("idle");
  });

  it("reports readiness inspection failures instead of treating them as unprepared", async () => {
    mocks.getLocalResourcesStatus.mockRejectedValueOnce(new Error("Unable to inspect resources"));
    scheduleSpaceKnowledgeIndex("test-space");
    await vi.advanceTimersByTimeAsync(5_000);

    expect(mocks.indexKnowledgeRetrieval).not.toHaveBeenCalled();
    expect(mocks.logError).toHaveBeenCalledOnce();
    expect(knowledgeIndexStatus("test-space")).toBe("unavailable");
  });

  it("logs structured AI errors instead of object stringification", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue({
      code: "provider_request_failed",
      message: "Embedding provider request failed",
    });

    scheduleKnowledgeIndex("structured-ai-error", ["node-1"]);
    await vi.runAllTimersAsync();

    expect(mocks.logError).toHaveBeenCalledWith(
      "Knowledge semantic indexing failed: [provider_request_failed] Embedding provider request failed",
    );
    expect(knowledgeIndexStatus("structured-ai-error")).toBe("unavailable");
  });

  it("retains the scope of a failed attempt and retries it with a growing delay", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValueOnce(new Error("deadline_exceeded"));

    scheduleKnowledgeIndex("test-space", ["node-1"]);
    await vi.advanceTimersByTimeAsync(5_000);

    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledExactlyOnceWith("test-space", ["node-1"]);
    expect(knowledgeIndexStatus("test-space")).toBe("unavailable");

    // The retry waits; indexing shares one worker and must not spin on a failing dependency.
    await vi.advanceTimersByTimeAsync(4_999);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledOnce();

    await vi.advanceTimersByTimeAsync(1);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenLastCalledWith("test-space", ["node-1"]);
    expect(knowledgeIndexStatus("test-space")).toBe("ready");
  });

  it("backs off further for every consecutive failure and recovers immediately on success", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue(new Error("deadline_exceeded"));

    scheduleSpaceKnowledgeIndex("test-space");
    await vi.advanceTimersByTimeAsync(5_000);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledOnce();

    await vi.advanceTimersByTimeAsync(5_000);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(9_999);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledTimes(2);

    mocks.indexKnowledgeRetrieval.mockResolvedValue({ indexed: 1, staleNodeIds: [] });
    await vi.advanceTimersByTimeAsync(1);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledTimes(3);
    // The whole-Space scope survived both failures, so the successful attempt still covers it.
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenLastCalledWith("test-space", undefined);
    expect(knowledgeIndexStatus("test-space")).toBe("ready");
  });
});
