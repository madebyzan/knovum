import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import type { VoiceCandidate } from "./contracts";
import { getVoiceCandidate } from "./voice-candidate-ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function observeVoiceCandidateRun(
  run: RunObservation,
  spaceId: string,
  succeeded: (candidate: VoiceCandidate) => void,
  failed: (message: string) => void,
) {
  const finished = (result: RunObservation) => {
    if (result.state !== "succeeded") {
      failed(i18n.global.t(workflowStateLabelKey(result.state)));
      return;
    }
    void getVoiceCandidate(result.id, spaceId)
      .then(succeeded)
      .catch((cause: unknown) => {
        failed(getErrorMessage(cause));
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
