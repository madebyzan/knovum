import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunEvent } from "../contracts/generated/RunEvent";

function eventFailure(event: RunEvent): string | undefined {
  if (event.kind !== "step_failed" || typeof event.payload !== "object" || event.payload === null)
    return undefined;
  if (Array.isArray(event.payload)) return undefined;
  if ("error" in event.payload) return getErrorMessage(event.payload.error);
  return typeof event.payload.reason === "string" ? event.payload.reason : undefined;
}

export async function loadLatestRunFailure(runId: string): Promise<string | undefined> {
  let afterSequence = 0;
  let latest: string | undefined;
  while (true) {
    // oxlint-disable-next-line no-await-in-loop -- Each page cursor comes from the preceding response, so requests cannot run in parallel.
    const events = await invokeCommand<RunEvent[]>("plugin:spaces|list_run_events", {
      runId,
      afterSequence,
    });
    for (const event of events) latest = eventFailure(event) ?? latest;
    const lastSequence = events.at(-1)?.sequence;
    if (events.length < 1000 || lastSequence === undefined || lastSequence <= afterSequence) return latest;
    afterSequence = lastSequence;
  }
}
