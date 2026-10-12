import { beforeEach, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke }));

import { loadLatestRunFailure } from "../run-failure";

beforeEach(() => bridge.invoke.mockReset());

it("returns the latest persisted step failure message", async () => {
  bridge.invoke.mockResolvedValue([
    {
      sequence: 1,
      runId: "run",
      stepInstanceId: "step",
      attemptId: "attempt-1",
      kind: "step_failed",
      payload: { error: { code: "conversion_failed", message: "First attempt failed" } },
      createdAt: 1,
    },
    {
      sequence: 2,
      runId: "run",
      stepInstanceId: "step",
      attemptId: "attempt-2",
      kind: "step_failed",
      payload: {
        error: { code: "document_conversion_failed", message: "Document converter could not open the file" },
      },
      createdAt: 2,
    },
  ]);

  await expect(loadLatestRunFailure("run")).resolves.toBe("Document converter could not open the file");
  expect(bridge.invoke).toHaveBeenCalledWith("plugin:spaces|list_run_events", {
    runId: "run",
    afterSequence: 0,
  });
});

it("continues through capped event pages before selecting the latest failure", async () => {
  const firstPage = Array.from({ length: 1000 }, (_, index) => ({
    sequence: index + 1,
    runId: "run",
    stepInstanceId: null,
    attemptId: null,
    kind: "run_queued",
    payload: {},
    createdAt: index + 1,
  }));
  bridge.invoke.mockResolvedValueOnce(firstPage).mockResolvedValueOnce([
    {
      sequence: 1001,
      runId: "run",
      stepInstanceId: "step",
      attemptId: "attempt",
      kind: "step_failed",
      payload: { error: { code: "conversion_failed", message: "Final conversion failure" } },
      createdAt: 1001,
    },
  ]);

  await expect(loadLatestRunFailure("run")).resolves.toBe("Final conversion failure");
  expect(bridge.invoke).toHaveBeenNthCalledWith(2, "plugin:spaces|list_run_events", {
    runId: "run",
    afterSequence: 1000,
  });
});
