import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { effectScope } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type { TranslateVideoDubbingRequest } from "../../contracts/generated/TranslateVideoDubbingRequest";
import type { VideoTranslation } from "../../contracts/generated/VideoTranslation";
import type * as runObservation from "../../runs/resource-run-observation";
import type * as ipc from "../ipc";
import { useVideoTranslationRun } from "../use-video-translation-run";

const mocks = vi.hoisted(() => ({
  cancelResourceRun: vi.fn<typeof runObservation.cancelResourceRun>(),
  getVideoTranslation: vi.fn<typeof ipc.getVideoTranslation>(),
  getVideoTranslationRun: vi.fn<typeof ipc.getVideoTranslationRun>(),
  observeResourceRun: vi.fn<typeof runObservation.observeResourceRun>(),
  translateVideoDubbing: vi.fn<typeof ipc.translateVideoDubbing>(),
}));
vi.mock("../ipc", () => mocks);
vi.mock("../../runs/resource-run-observation", () => ({
  cancelResourceRun: mocks.cancelResourceRun,
  observeResourceRun: mocks.observeResourceRun,
}));
vi.mock("../../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));

const request: TranslateVideoDubbingRequest = {
  jobId: "run",
  projectId: "project",
  expectedRevision: 1,
  targetLanguage: "en",
  localOnly: false,
  executionTarget: { type: "model", connectionId: "connection", modelId: "model", temperature: 0 },
  segments: [{ sourceSegmentId: "line", sourceSegmentRevision: 1, text: "你好" }],
};
const completed: RunObservation = {
  id: "run",
  homeSpaceId: "space",
  parentRunId: null,
  retryOfRunId: null,
  forkOfRunId: null,
  kind: "workflow",
  state: "succeeded",
  stateVersion: 1,
  waitReason: null,
  inputTokens: 0,
  outputTokens: 0,
  totalTokens: 0,
  stepAttempts: 1,
  expandedSteps: 1,
  outputBytes: 0,
  createdAt: 1,
  updatedAt: 1,
  completedAt: 1,
};
function deferred<T>() {
  let complete!: (value: T) => void;
  const promise = new Promise<T>((resolve) => {
    complete = resolve;
  });
  return { promise, resolve: complete };
}
function setup() {
  const scope = effectScope();
  const apply = vi.fn<(translation: VideoTranslation) => void>();
  const report = vi.fn<(message: string) => void>();
  const translation = scope.run(() => useVideoTranslationRun(apply, report))!;
  onTestFinished(() => scope.stop());
  return { translation, apply, report };
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.getVideoTranslationRun.mockResolvedValue(null);
  mocks.observeResourceRun.mockReturnValue(vi.fn<() => undefined>());
});

it("does not start a duplicate translation while looking up the existing durable run", async () => {
  const lookup = deferred<RunObservation | null>();
  mocks.getVideoTranslationRun.mockReturnValue(lookup.promise);
  const { translation } = setup();
  const restoring = translation.restore("project", 1);
  await translation.start(request);
  expect(mocks.translateVideoDubbing).not.toHaveBeenCalled();
  lookup.resolve(null);
  await restoring;
  expect(translation.submitting.value).toBe(false);
});

it("clears the old submitting state when the project revision changes during creation", async () => {
  const creation = deferred<RunObservation>();
  mocks.translateVideoDubbing.mockReturnValue(creation.promise);
  const { translation, apply } = setup();
  const starting = translation.start(request);
  await translation.restore("project", 2);
  creation.resolve(completed);
  await starting;
  expect(translation.submitting.value).toBe(false);
  expect(mocks.getVideoTranslation).not.toHaveBeenCalled();
  expect(apply).not.toHaveBeenCalled();
});

it("keeps the script busy until the completed run's result has arrived", async () => {
  mocks.getVideoTranslationRun.mockResolvedValueOnce(completed).mockResolvedValueOnce(null);
  const output = deferred<VideoTranslation>();
  mocks.getVideoTranslation.mockReturnValue(output.promise);
  const { translation } = setup();
  const restoring = translation.restore("project", 1);
  await vi.waitFor(() => expect(mocks.getVideoTranslation).toHaveBeenCalledWith("run"));
  await translation.start(request);
  expect(mocks.translateVideoDubbing).not.toHaveBeenCalled();
  await translation.restore("other-project", 1);
  output.resolve({
    segments: [],
    provenance: {
      runId: "run",
      targetLanguage: "en",
      localOnly: false,
      provider: "connection",
      model: "model",
    },
  });
  await restoring;
  expect(translation.submitting.value).toBe(false);
});

it("settles cancellation without presenting it as a translation failure", async () => {
  const running = { ...completed, state: "running" as const, completedAt: null };
  mocks.getVideoTranslationRun.mockResolvedValue(running);
  const { translation, report } = setup();
  await translation.restore("project", 1);
  mocks.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state: "cancelled" });
  expect(translation.activeRunId.value).toBeUndefined();
  expect(report).not.toHaveBeenCalled();
});

it("keeps cancellation failures attached to the active translation", async () => {
  const running = { ...completed, state: "running" as const, completedAt: null };
  mocks.getVideoTranslationRun.mockResolvedValue(running);
  mocks.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
  const { translation, report } = setup();
  await translation.restore("project", 1);
  await translation.cancel();
  expect(report).toHaveBeenCalledWith("Cancellation denied");
  expect(translation.activeRunId.value).toBe("run");
  expect(translation.cancelling.value).toBe(false);
});
