import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";

interface RunObserver {
  update(run: RunObservation): void;
  error(cause: unknown): void;
  finished(): void;
}
const observations = new Map<string, Set<RunObserver>>();

export function observeRun(runId: string, observer: RunObserver) {
  const current = observations.get(runId);
  if (current) {
    current.add(observer);
    return;
  }
  const observers = new Set([observer]);
  observations.set(runId, observers);
  const updates = new Channel<RunObservation>((run) => {
    for (const subscriber of observers) subscriber.update(run);
  });
  void invokeCommand("plugin:spaces|observe_run", { runId, updates })
    .catch((cause: unknown) => {
      for (const subscriber of observers) subscriber.error(cause);
    })
    .finally(() => {
      observations.delete(runId);
      for (const subscriber of observers) subscriber.finished();
    });
}
