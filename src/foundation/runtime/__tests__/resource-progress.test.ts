import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import type { ResourcePreparationProgress } from "../../../shared/contracts/generated/ResourcePreparationProgress";
import { useTaskStore } from "../../tasks/task-store";
import { updateTaskProgress } from "../resource-progress";

vi.mock("../../i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
beforeEach(() => setActivePinia(createPinia()));

it("keeps an unknown-size runtime transfer visible without inventing a total or failing the task", () => {
  const tasks = useTaskStore();
  const id = tasks.start("Download");
  updateTaskProgress(tasks, id, {
    stage: "runtimeDownloading",
    downloaded: 1024,
    total: null,
    percent: null,
  });
  expect(tasks.tasks[0]?.status).toBe("running");
  expect(tasks.tasks[0]?.progress).toBeUndefined();
  expect(tasks.tasks[0]?.statusMessage).toContain("1.0 KiB");
  expect(tasks.tasks[0]?.statusMessage).not.toContain("/");
});

it.each(["verifying", "runtimeVerifying", "installing", "runtimeInstalling"] as const)(
  "distinguishes %s from completed downloading without a misleading 100 percent",
  (stage) => {
    const tasks = useTaskStore();
    const id = tasks.start("Prepare");
    const progress: ResourcePreparationProgress = { stage, downloaded: 100, total: 100, percent: 100 };
    updateTaskProgress(tasks, id, progress);
    expect(tasks.tasks[0]?.status).toBe("running");
    expect(tasks.tasks[0]?.progress).toBeUndefined();
    expect(tasks.tasks[0]?.statusMessage).toContain(
      stage.includes("erifying") ? "tasks.verifyingResource" : "tasks.installingResource",
    );
  },
);
