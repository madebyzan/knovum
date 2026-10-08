import { beforeEach, expect, it, vi } from "vitest";

import type { DraftVoiceCandidateRequest } from "../../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../../contracts/generated/RunObservation";
import type * as observation from "../../runs/resource-run-observation";
import type * as speechIpc from "../ipc";
import { useVoiceCandidateSubmission } from "../use-voice-candidate-submission";
import type * as candidateIpc from "../voice-candidate-ipc";

const mocks = vi.hoisted(() => ({
  draftVoiceCandidate: vi.fn<typeof candidateIpc.draftVoiceCandidate>(),
  cancelSpeechGeneration: vi.fn<typeof speechIpc.cancelSpeechGeneration>(),
  observeResourceRun: vi.fn<typeof observation.observeResourceRun>(),
  cancelResourceRun: vi.fn<typeof observation.cancelResourceRun>(),
}));
vi.mock("../voice-candidate-ipc", () => mocks);
vi.mock("../ipc", () => mocks);
vi.mock("../../runs/resource-run-observation", () => mocks);
vi.mock("../../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));

const input: Omit<DraftVoiceCandidateRequest, "requestId"> = {
  spaceId: "space",
  model: "voxCpm2",
  language: "en",
  text: "Example",
  seed: 0,
  authorized: true,
  source: { kind: "designed", description: "Warm narrator" },
};
const run: RunObservation = {
  id: "created-run",
  homeSpaceId: "space",
  kind: "workflow",
  state: "running",
  stateVersion: 1,
  parentRunId: null,
  retryOfRunId: null,
  forkOfRunId: null,
  waitReason: null,
  inputTokens: 0,
  outputTokens: 0,
  totalTokens: 0,
  stepAttempts: 1,
  expandedSteps: 1,
  outputBytes: 0,
  createdAt: 1,
  updatedAt: 1,
  completedAt: null,
};
beforeEach(() => vi.resetAllMocks());

it.each([false, true])(
  "retains cancellation while task creation settles, including closed view: %s",
  async (closed) => {
    let created!: (value: RunObservation) => void;
    mocks.draftVoiceCandidate.mockReturnValue(
      new Promise((resolve) => {
        created = resolve;
      }),
    );
    const submission = useVoiceCandidateSubmission();
    const attach = vi.fn<(run: RunObservation) => void>();
    const completion = submission.start(input, attach);
    const jobId = submission.jobId.value;
    await submission.cancel();
    expect(mocks.cancelSpeechGeneration).toHaveBeenCalledExactlyOnceWith(jobId);
    if (closed) submission.reset();
    created(run);
    await completion;
    expect(mocks.observeResourceRun).toHaveBeenCalledExactlyOnceWith(run, "spaces.speech.generateCandidate");
    expect(attach).toHaveBeenCalledExactlyOnceWith(run);
    expect(mocks.cancelResourceRun).toHaveBeenCalledExactlyOnceWith(run.id);
    expect(submission.jobId.value).toBeUndefined();
  },
);

it("attaches the created task before surfacing a persistent cancellation failure", async () => {
  let created!: (value: RunObservation) => void;
  mocks.draftVoiceCandidate.mockReturnValue(
    new Promise((resolve) => {
      created = resolve;
    }),
  );
  mocks.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
  const submission = useVoiceCandidateSubmission();
  const attach = vi.fn<(run: RunObservation) => void>();
  const completion = submission.start(input, attach);
  await submission.cancel();
  created(run);
  await expect(completion).rejects.toThrow("Cancellation denied");
  expect(attach).toHaveBeenCalledExactlyOnceWith(run);
  expect(submission.jobId.value).toBeUndefined();
});

it("does not clear a newer submission when a detached creation settles", async () => {
  let oldCreated!: (value: RunObservation) => void;
  mocks.draftVoiceCandidate.mockReturnValueOnce(
    new Promise((resolve) => {
      oldCreated = resolve;
    }),
  );
  mocks.draftVoiceCandidate.mockReturnValueOnce(new Promise(() => undefined));
  const submission = useVoiceCandidateSubmission();
  const old = submission.start(input, vi.fn<(run: RunObservation) => void>());
  submission.reset();
  void submission.start({ ...input, spaceId: "second-space" }, vi.fn<(run: RunObservation) => void>());
  const newId = submission.jobId.value;
  oldCreated(run);
  await old;
  expect(submission.jobId.value).toBe(newId);
  expect(mocks.cancelResourceRun).not.toHaveBeenCalled();
});

it("does not create a phantom durable task when preparation is cancelled", async () => {
  const cancellation = new Error("cancelled");
  mocks.draftVoiceCandidate.mockRejectedValue(cancellation);
  const submission = useVoiceCandidateSubmission();
  const attach = vi.fn<(run: RunObservation) => void>();
  await expect(submission.start(input, attach)).rejects.toBe(cancellation);
  expect(mocks.observeResourceRun).not.toHaveBeenCalled();
  expect(attach).not.toHaveBeenCalled();
  expect(submission.jobId.value).toBeUndefined();
});
