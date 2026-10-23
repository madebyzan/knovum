import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { KnowledgeIndexingProgress } from "../../../contracts/generated/KnowledgeIndexingProgress";
import type { KnowledgeIndexingInput } from "../../persistence/retrieval-index";

const mocks = vi.hoisted(() => ({
  indexKnowledgeRetrieval: vi.fn<(input: KnowledgeIndexingInput) => Promise<{ indexed: number }>>(),
  cancelKnowledgeIndexing: vi.fn<(spaceId: string) => Promise<void>>(),
}));

vi.mock("../../persistence/retrieval-index", () => ({
  indexKnowledgeRetrieval: mocks.indexKnowledgeRetrieval,
  cancelKnowledgeIndexing: mocks.cancelKnowledgeIndexing,
}));

import { createAppError } from "../../../../../foundation/errors/app-error";
import { useTaskStore } from "../../../../../foundation/tasks/task-store";
import { isKnowledgeIndexingCancelled } from "../indexing-outcome";
import { knowledgeIndexingTaskId, startKnowledgeIndexing } from "../start-indexing";

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
    const handle = startKnowledgeIndexing("space-progress", "Rebuilding the index");
    const tasks = useTaskStore();
    const task = () =>
      tasks.tasks.find((candidate) => candidate.id === knowledgeIndexingTaskId("space-progress"));

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

  it("joins the pass a Space already has instead of opening a second one", async () => {
    const pass = streamingPass();
    const first = startKnowledgeIndexing("space-join", "Rebuilding the index");
    const second = startKnowledgeIndexing("space-join", "Enabling semantic search");

    // A Space has one pass, so it has one task, one cancel button and one answer.
    expect(second).toBe(first);
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledOnce();
    expect(useTaskStore().tasks).toHaveLength(1);

    pass.finish(3);
    await expect(second.completion).resolves.toEqual({ indexed: 3 });

    // The settled pass releases the Space: the next request is a new pass with a new task.
    streamingPass();
    startKnowledgeIndexing("space-join", "Rebuilding the index");
    expect(mocks.indexKnowledgeRetrieval).toHaveBeenCalledTimes(2);
  });

  it("cancels the Space's pass, because that is what a pass belongs to", async () => {
    const pass = streamingPass();
    const handle = startKnowledgeIndexing("space-cancel", "Rebuilding the index");

    await handle.cancel();

    expect(mocks.cancelKnowledgeIndexing).toHaveBeenCalledExactlyOnceWith("space-cancel");
    pass.finish(0);
    await handle.completion;
  });

  it("settles a cancelled pass as cancelled rather than failed", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue(
      createAppError("cancelled", "Knowledge indexing was cancelled"),
    );

    const handle = startKnowledgeIndexing("space-stopped", "Rebuilding the index");
    await expect(handle.completion).rejects.toSatisfy(isKnowledgeIndexingCancelled);

    expect(useTaskStore().tasks[0]?.status).toBe("cancelled");
  });

  it("settles a real failure as failed", async () => {
    mocks.indexKnowledgeRetrieval.mockRejectedValue(new Error("Embedding worker is unavailable"));

    const handle = startKnowledgeIndexing("space-failed", "Rebuilding the index");
    await expect(handle.completion).rejects.toThrow("Embedding worker is unavailable");

    expect(useTaskStore().tasks[0]?.status).toBe("failed");
  });
});
