import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { refreshEnrichmentTasks } from "../job-observation";

const jobs = vi.hoisted(() => ({
  cancel: vi.fn<(jobId: string) => Promise<void>>(),
  list: vi.fn<(spaceId: string) => Promise<unknown[]>>(),
  run: vi.fn<(job: unknown, onProgress?: (progress: unknown) => void) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../enrichment-jobs", () => ({
  cancelEnrichmentJob: jobs.cancel,
  listEnrichmentJobs: jobs.list,
  retryEnrichmentJob: vi.fn<(jobId: string) => Promise<void>>(),
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

describe("space enrichment observation", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("observes a persisted pending job without dispatching work in the renderer", async () => {
    jobs.list.mockResolvedValue([{ ...record, status: "pending" }]);
    jobs.run.mockResolvedValue({ nodeId: "node-1", data: { kind: "audio", title: "Recording" } });
    const applyNodeContentForSpace = vi.fn<(spaceId: string, nodeId: string, data: unknown) => void>();

    await refreshEnrichmentTasks("space-1");

    expect(jobs.run).not.toHaveBeenCalled();
    expect(applyNodeContentForSpace).not.toHaveBeenCalled();
    expect(useTaskStore().tasks[0]).toMatchObject({ id: "job-1", status: "running" });
  });

  it("restores visual analysis jobs through the shared task projection", async () => {
    jobs.list.mockResolvedValue([
      { ...record, id: "image-job", jobType: "analyze_visual", status: "pending" },
      { ...record, id: "cover-job", nodeId: "video-node", jobType: "analyze_visual", status: "running" },
    ]);

    await refreshEnrichmentTasks("space-1");

    const tasks = useTaskStore().tasks;
    expect(tasks).toHaveLength(2);
    expect(tasks.every((task) => task.status === "running")).toBe(true);
    expect(tasks[0]?.title).toBe(tasks[1]?.title);
    expect(tasks[0]?.title).not.toContain("spaces.processing.enrichmentJobType");
    expect(jobs.run).not.toHaveBeenCalled();
  });

  it("restores a persisted failure with its real error and retry action", async () => {
    jobs.list.mockResolvedValue([
      { ...record, status: "failed", error: "Required Whisper model is not installed" },
    ]);

    await refreshEnrichmentTasks("space-1");

    expect(useTaskStore().tasks[0]).toMatchObject({
      id: "job-1",
      status: "failed",
      detail: "Required Whisper model is not installed",
      retryable: true,
    });
  });
});
