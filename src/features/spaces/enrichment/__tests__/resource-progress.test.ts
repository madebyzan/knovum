import { createPinia, setActivePinia } from "pinia";
import { expect, it, onTestFinished, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { useTaskStore } from "../../../../foundation/tasks/task-store";
import type { EnrichmentJobUpdate } from "../../contracts/generated/EnrichmentJobUpdate";
import { ENRICHMENT_TASK_PRODUCER } from "../job-tasks";
import { useEnrichmentJobObservation } from "../use-enrichment-job-observation";

const mocks = vi.hoisted(() => ({
  observe: undefined as ((event: { payload: EnrichmentJobUpdate }) => void) | undefined,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn<
    (name: string, callback: (event: { payload: EnrichmentJobUpdate }) => void) => Promise<() => void>
  >(async (name, callback) => {
    if (name === "space-enrichment-update") mocks.observe = callback;
    return () => {};
  }),
}));
vi.mock("../enrichment-jobs-query", () => ({ refreshEnrichmentJobs: vi.fn<() => Promise<void>>() }));

it("projects the whole resource download budget onto the cancellable node task", () => {
  setActivePinia(createPinia());
  const tasks = useTaskStore();
  const cancel = vi.fn<() => Promise<void>>();
  tasks.reconcileObserved(ENRICHMENT_TASK_PRODUCER, [
    {
      task: { id: "job", title: "Speaker recognition", startedAt: 1, status: "running" },
      cancel,
    },
  ]);
  const scope = effectScope();
  scope.run(() =>
    useEnrichmentJobObservation({
      activeSpaceId: shallowRef("space"),
      analysisIds: shallowRef([]),
      analysisPreviews: shallowRef({}),
      applyNodeContentForSpace: vi.fn<() => void>(),
    }),
  );
  onTestFinished(() => scope.stop());
  mocks.observe!({
    payload: {
      jobId: "job",
      spaceId: "space",
      nodeId: "node",
      jobType: "diarize_media_transcript",
      createdAt: 1,
      status: "running",
      data: null,
      error: null,
      progress: {
        stage: "preparingResources",
        progress: 0,
        partial: null,
        resources: { stage: "runtimeDownloading", downloaded: 512, total: 1024, percent: 50 },
      },
    },
  });
  expect(tasks.tasks[0]).toMatchObject({ status: "running", progress: 50, cancellable: true });
  expect(tasks.tasks[0]!.statusMessage).toContain("512 B / 1.0 KiB");
  expect(cancel).not.toHaveBeenCalled();
});
