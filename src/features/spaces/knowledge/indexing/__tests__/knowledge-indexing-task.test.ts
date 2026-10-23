import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { KnowledgeIndexingProgress } from "../../../contracts/generated/KnowledgeIndexingProgress";
import type { KnowledgeIndexingInput } from "../../persistence/retrieval-index";

const mocks = vi.hoisted(() => ({
  indexKnowledgeRetrieval: vi.fn<(input: KnowledgeIndexingInput) => Promise<{ indexed: number }>>(),
  cancelKnowledgeIndexing: vi.fn<(jobId: string) => Promise<void>>(),
}));

vi.mock("../../persistence/retrieval-index", () => ({
  indexKnowledgeRetrieval: mocks.indexKnowledgeRetrieval,
  cancelKnowledgeIndexing: mocks.cancelKnowledgeIndexing,
}));

import { createAppError } from "../../../../../foundation/errors/app-error";
import { useTaskStore } from "../../../../../foundation/tasks/task-store";
import { isKnowledgeIndexingCancelled, startKnowledgeIndexing } from "../start-indexing";

/** Holds the pass open so the test can stream progress into the live task. */
function streamingPass() {
  const state: {
    report: (progress: KnowledgeIndexingProgress) => void;
    finish: (output: { indexed: number }) => void;
  } = { report: () => {}, finish: () => {} };
  mocks.indexKnowledgeRetrieval.mockImplementation((input: KnowledgeIndexingInput) => {
    state.report = input.onProgress;
    return new Promise((resolve) => {
      state.finish = resolve;
    });
  });
  return {
    report: (progress: KnowledgeIndexingProgress) => state.report(progress),
    finish: (indexed: number) => state.finish({ indexed }),
  };
}

describe("knowledge indexing task", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.indexKnowledgeRetrieval.mockReset();
    mocks.cancelKnowledgeIndexing.mockReset().mockResolvedValue();
  });

  it("shows one cancellable task carrying the share of the work the pass found", async () => {
    const pass = streamingPass();
    const handle = startKnowledgeIndexing("space-1", "Rebuilding the index");
    const tasks = useTaskStore();
    const task = () => tasks.tasks.find((candidate) => candidate.title === "Rebuilding the index");

    expect(task()?.cancellable).toBe(true);
    pass.report({ indexed: 25, total: 100 });
    expect(task()?.progress).toBe(25);

    // A Space written to while it indexes feeds the same pass more chunks, so the measured count
    // can pass the count taken at the start. The ratio a person sees stays bounded.
    pass.report({ indexed: 140, total: 100 });
    expect(task()?.progress).toBe(100);

    pass.finish(140);
    await handle.completion;
    expect(task()?.status).toBe("succeeded");
  });

  it("cancels the job it started", async () => {
    const pass = streamingPass();
    const handle = startKnowledgeIndexing("space-1", "Rebuilding the index");

    await handle.cancel();

    const started = mocks.indexKnowledgeRetrieval.mock.calls[0]?.[0] as KnowledgeIndexingInput;
    expect(mocks.cancelKnowledgeIndexing).toHaveBeenCalledExactlyOnceWith(started.jobId);
    pass.finish(0);
    await handle.completion;
  });

  it("settles a cancelled pass as cancelled rather than failed", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue(
      createAppError("cancelled", "Knowledge indexing was cancelled"),
    );

    const handle = startKnowledgeIndexing("space-1", "Rebuilding the index");
    await expect(handle.completion).rejects.toSatisfy(isKnowledgeIndexingCancelled);

    expect(useTaskStore().tasks[0]?.status).toBe("cancelled");
  });

  it("settles a real failure as failed", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue(new Error("Embedding worker is unavailable"));

    const handle = startKnowledgeIndexing("space-1", "Rebuilding the index");
    await expect(handle.completion).rejects.toThrow("Embedding worker is unavailable");

    expect(useTaskStore().tasks[0]?.status).toBe("failed");
  });
});
