import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import type { ResourcePreparationProgress } from "../../../shared/contracts/generated/ResourcePreparationProgress";
import { useTaskStore } from "../../tasks/task-store";
import { buildResourcePreparationTask, updateTaskProgress } from "../resource-progress";

vi.mock("../../i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
beforeEach(() => setActivePinia(createPinia()));

it("keeps an unknown-size resource transfer visible without inventing a total or failing the task", () => {
  const tasks = useTaskStore();
  const id = tasks.start("Download");
  updateTaskProgress(tasks, id, {
    stage: "downloading",
    downloaded: 1024,
    total: null,
    percent: null,
  });
  expect(tasks.tasks[0]?.status).toBe("running");
  expect(tasks.tasks[0]?.progress).toBeUndefined();
  expect(tasks.tasks[0]?.statusMessage).toContain("1.0 KiB");
  expect(tasks.tasks[0]?.statusMessage).not.toContain("/");
});

it.each(["verifying", "installing"] as const)(
  "distinguishes %s from completed downloading without a misleading 100 percent",
  (stage) => {
    const tasks = useTaskStore();
    const id = tasks.start("Prepare");
    const progress: ResourcePreparationProgress = { stage, downloaded: 100, total: 100, percent: 100 };
    updateTaskProgress(tasks, id, progress);
    expect(tasks.tasks[0]?.status).toBe("running");
    expect(tasks.tasks[0]?.progress).toBeUndefined();
    expect(tasks.tasks[0]?.statusMessage).toContain(
      stage.includes("erifying") ? "tasks.resources.verifying" : "tasks.resources.installing",
    );
  },
);

it("restores verification without projecting its completed transfer as task completion", () => {
  const task = buildResourcePreparationTask({
    jobId: "prepare-model",
    title: "Model",
    target: { kind: "model", model: "whisperSmall" },
    startedAt: 100,
    completedAt: null,
    state: "preparing",
    error: null,
    progress: { stage: "verifying", downloaded: 100, total: 100, percent: 100 },
  });
  expect(task.progress).toBeUndefined();
});
