import { beforeEach, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import { runDerivation } from "../knowledge-derivations/run-derivation";

const state = vi.hoisted(() => ({
  toastError: vi.fn<(message: string) => void>(),
  tasks: {
    start: vi.fn<ReturnType<typeof useTaskStore>["start"]>(),
    succeed: vi.fn<ReturnType<typeof useTaskStore>["succeed"]>(),
    fail: vi.fn<ReturnType<typeof useTaskStore>["fail"]>(),
  },
}));
vi.mock("vue-sonner", () => ({ toast: { error: state.toastError } }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => state.tasks }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../knowledge-derivations/canvas-yield", () => ({ yieldToCanvas: async () => undefined }));

beforeEach(() => {
  vi.clearAllMocks();
  state.tasks.start.mockReturnValue("task");
});

it("settles a failed task even if output recovery fails", async () => {
  const onError = vi.fn<() => Promise<void>>().mockRejectedValue(new Error("Recovery failed"));
  await expect(
    runDerivation({
      store: { activeSpaceId: "space" },
      pendingId: "output",
      task: { title: "Derive" },
      failureMessageKey: "derivation.failed",
      execute: async () => {
        throw new Error("Derivation failed");
      },
      onSuccess: () => undefined,
      onError,
    }),
  ).rejects.toMatchObject({
    name: "AggregateError",
    cause: expect.objectContaining({ message: "Derivation failed" }),
    errors: [
      expect.objectContaining({ message: "Derivation failed" }),
      expect.objectContaining({ message: "Recovery failed" }),
    ],
  });
  expect(state.tasks.fail).toHaveBeenCalledWith("task", expect.stringContaining("Derivation failed"));
  expect(state.toastError).not.toHaveBeenCalled();
});

it("does not run failure recovery after a committed output has a projection error", async () => {
  const onError = vi.fn<() => Promise<void>>();
  await expect(
    runDerivation({
      store: { activeSpaceId: "space" },
      pendingId: "output",
      task: { title: "Derive" },
      failureMessageKey: "derivation.failed",
      execute: async () => "committed",
      onSuccess: async () => {
        throw new Error("Projection failed");
      },
      onError,
    }),
  ).rejects.toThrow("Projection failed");
  expect(onError).not.toHaveBeenCalled();
  expect(state.tasks.succeed).toHaveBeenCalledWith("task");
  expect(state.tasks.fail).not.toHaveBeenCalled();
  expect(state.toastError).not.toHaveBeenCalled();
});
