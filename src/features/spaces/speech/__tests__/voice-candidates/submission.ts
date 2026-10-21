import { beforeEach, describe, expect, it, vi } from "vitest";

import type { DraftVoiceCandidateRequest } from "../../../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../../../contracts/generated/RunObservation";
import { useVoiceCandidateSubmission } from "../../use-voice-candidate-submission";
import { runningRun, voiceHost } from "../support/voice-candidate-host";

/** Submitting a candidate draft and settling its run. */
export function submissionScenarios() {
  describe("voice candidate submission", () => {
    const input: Omit<DraftVoiceCandidateRequest, "requestId"> = {
      model: "voxCpm2",
      language: "en",
      text: "Example",
      seed: 0,
      source: { kind: "designed", description: "Warm narrator" },
    };
    const run = runningRun("created-run");
    beforeEach(() => vi.resetAllMocks());

    it("retains cancellation while task creation settles", async () => {
      let created!: (value: RunObservation) => void;
      voiceHost.draftVoiceCandidate.mockReturnValue(
        new Promise((resolve) => {
          created = resolve;
        }),
      );
      const submission = useVoiceCandidateSubmission();
      const completion = submission.start(input);
      const jobId = submission.jobId.value;
      await submission.cancel();
      expect(voiceHost.cancelSpeechGeneration).toHaveBeenCalledExactlyOnceWith(jobId);
      created(run);
      await expect(completion).resolves.toBe(run);
      expect(voiceHost.observeResourceRun).toHaveBeenCalledExactlyOnceWith(
        run,
        "spaces.speech.voice.candidate.generate",
      );
      expect(voiceHost.cancelResourceRun).toHaveBeenCalledExactlyOnceWith(run.id);
      expect(submission.jobId.value).toBeUndefined();
    });

    it("observes the created task before surfacing a persistent cancellation failure", async () => {
      let created!: (value: RunObservation) => void;
      voiceHost.draftVoiceCandidate.mockReturnValue(
        new Promise((resolve) => {
          created = resolve;
        }),
      );
      voiceHost.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
      const submission = useVoiceCandidateSubmission();
      const completion = submission.start(input);
      await submission.cancel();
      created(run);
      await expect(completion).rejects.toThrow("Cancellation denied");
      expect(voiceHost.observeResourceRun).toHaveBeenCalledOnce();
      expect(submission.jobId.value).toBeUndefined();
    });

    it("does not create a phantom durable task when preparation is cancelled", async () => {
      const cancellation = new Error("cancelled");
      voiceHost.draftVoiceCandidate.mockRejectedValue(cancellation);
      const submission = useVoiceCandidateSubmission();
      await expect(submission.start(input)).rejects.toBe(cancellation);
      expect(voiceHost.observeResourceRun).not.toHaveBeenCalled();
      expect(submission.jobId.value).toBeUndefined();
    });
  });
}
