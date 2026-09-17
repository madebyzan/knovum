import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  indexKnowledgeRetrieval:
    vi.fn<
      (spaceId: string, nodeIds?: readonly string[]) => Promise<{ indexed: number; staleNodeIds: string[] }>
    >(),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../knowledge/persistence/embedding-repository", () => ({
  indexKnowledgeRetrieval: mocks.indexKnowledgeRetrieval,
}));

import { knowledgeIndexStatus, scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";

describe("spaces knowledge index scheduler", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    mocks.indexKnowledgeRetrieval.mockReset();
    mocks.logError.mockReset().mockResolvedValue(undefined);
  });

  afterEach(() => vi.useRealTimers());

  it("logs structured AI errors instead of object stringification", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue({
      code: "provider_request_failed",
      message: "Embedding provider request failed",
    });

    scheduleKnowledgeIndex("structured-ai-error", ["node-1"]);
    await vi.runAllTimersAsync();

    expect(mocks.logError).toHaveBeenCalledWith(
      'Knowledge semantic indexing failed: {"code":"provider_request_failed","message":"Embedding provider request failed"}',
    );
    expect(knowledgeIndexStatus("structured-ai-error")).toBe("unavailable");
  });
});
