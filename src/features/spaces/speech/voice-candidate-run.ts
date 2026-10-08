import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { VoiceCandidate } from "../contracts/generated/VoiceCandidate";
import { observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { getVoiceCandidate } from "./voice-candidate-ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function observeVoiceCandidateRun(
  run: RunObservation,
  spaceId: string,
  succeeded: (candidate: VoiceCandidate) => void,
  settled: (error?: string) => void,
) {
  const finished = (observation: RunObservation) => {
    if (observation.state !== "succeeded") {
      settled(
        observation.state === "cancelled"
          ? undefined
          : i18n.global.t(workflowStateLabelKey(observation.state)),
      );
      return;
    }
    void getVoiceCandidate(observation.id, spaceId)
      .then(succeeded)
      .catch((cause: unknown) => {
        settled(getErrorMessage(cause));
      });
  };
  if (TERMINAL_STATES.has(run.state)) {
    finished(run);
    return undefined;
  }
  return observeResourceRun(run, i18n.global.t("spaces.speech.generateCandidate"), (next) => {
    if (TERMINAL_STATES.has(next.state)) finished(next);
  });
}
