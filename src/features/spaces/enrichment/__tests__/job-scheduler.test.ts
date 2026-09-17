import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { resumePendingEnrichmentJobs } from "../job-scheduler";

const jobs = vi.hoisted(() => ({
  cancel: vi.fn<(jobId: string) => Promise<void>>(),
  list: vi.fn<(spaceId: string) => Promise<unknown[]>>(),
  run: vi.fn<(job: unknown, onProgress?: (progress: unknown) => void) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../enrichment-jobs", () => ({
  cancelEnrichmentJob: jobs.cancel,
  listEnrichmentJobs: jobs.list,
  runEnrichmentJob: jobs.run,
}));

const record = {
  id: "job-1",
  spaceId: "space-1",
  nodeId: "node-1",
  jobType: "transcribe_media" as const,
  sourceRevision: "revision-1",
  attempts: 0,
  error: null,
  createdAt: 1,
  updatedAt: 1,
};

describe("space enrichment job scheduler", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("starts a persisted pending job as soon as the scheduler is resumed", async () => {
    jobs.list.mockResolvedValue([{ ...record, status: "pending" }]);
    jobs.run.mockResolvedValue({ nodeId: "node-1", data: { kind: "audio", title: "Recording" } });
    const applyNodeDataForSpace = vi.fn<(spaceId: string, nodeId: string, data: unknown) => void>();

    await resumePendingEnrichmentJobs({ applyNodeDataForSpace }, "space-1");

    await vi.waitFor(() =>
      expect(jobs.run).toHaveBeenCalledWith(expect.objectContaining({ id: "job-1" }), expect.any(Function)),
    );
    await vi.waitFor(() => expect(applyNodeDataForSpace).toHaveBeenCalledOnce());
  });

  it("restores a persisted failure with its real error and retry action", async () => {
    jobs.list.mockResolvedValue([
      { ...record, status: "failed", error: "Required Whisper model is not installed" },
    ]);

    await resumePendingEnrichmentJobs(
      { applyNodeDataForSpace: vi.fn<(spaceId: string, nodeId: string, data: unknown) => void>() },
      "space-1",
    );

    expect(useTaskStore().tasks[0]).toMatchObject({
      id: "job-1",
      status: "failed",
      detail: "Required Whisper model is not installed",
      retryable: true,
    });
  });
});
