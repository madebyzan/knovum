import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";

interface RunObserver {
  update(run: RunObservation): void;
  error(cause: unknown): void;
  finished(): void;
}
const observersByRunId = new Map<string, Set<RunObserver>>();

export function observeRun(runId: string, observer: RunObserver) {
  const current = observersByRunId.get(runId);
  if (current) {
    current.add(observer);
    return;
  }
  const observers = new Set([observer]);
  observersByRunId.set(runId, observers);
  const updates = new Channel<RunObservation>((run) => {
    for (const subscriber of observers) subscriber.update(run);
  });
  void invokeCommand("plugin:spaces|observe_run", { runId, updates })
    .catch((cause: unknown) => {
      for (const subscriber of observers) subscriber.error(cause);
    })
    .finally(() => {
      observersByRunId.delete(runId);
      for (const subscriber of observers) subscriber.finished();
    });
}
