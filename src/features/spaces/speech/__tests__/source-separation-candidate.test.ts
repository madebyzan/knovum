import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import type * as runObservation from "../../runs/resource-run-observation";
import type * as sourceSeparationIpc from "../source-separation-ipc";
import { useSourceSeparationCandidate } from "../use-source-separation-candidate";

const mocks = vi.hoisted(() => ({
  getLatestSourceSeparationRun: vi.fn<typeof sourceSeparationIpc.getLatestSourceSeparationRun>(),
  getSpeechSourceSeparation: vi.fn<typeof sourceSeparationIpc.getSpeechSourceSeparation>(),
  reviewSpeechSourceSeparation: vi.fn<typeof sourceSeparationIpc.reviewSpeechSourceSeparation>(),
  startSpeechSourceSeparation: vi.fn<typeof sourceSeparationIpc.startSpeechSourceSeparation>(),
  observeResourceRun: vi.fn<typeof runObservation.observeResourceRun>(),
  cancelResourceRun: vi.fn<typeof runObservation.cancelResourceRun>(),
}));
vi.mock("../source-separation-ipc", () => mocks);
vi.mock("../../runs/resource-run-observation", () => mocks);
vi.mock("../../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
const source = { assetPath: "/media.wav", sourceRevision: "sha256:source", durationMs: 1_000 };
const candidate: SpeechSourceSeparationCandidate = {
  separationId: "run",
  runId: "run",
  spaceId: "space",
  source,
  background: { ...source, assetPath: "/background.wav" },
  vocals: { ...source, assetPath: "/vocals.wav" },
  modelSpecId: "spleeter",
  reviewStatus: "pending",
  reviewWarning: "review",
};
function run(state: RunObservation["state"]): RunObservation {
  return {
    id: "run",
    homeSpaceId: "space",
    parentRunId: null,
    retryOfRunId: null,
    forkOfRunId: null,
    kind: "workflow",
    state,
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
    completedAt: state === "succeeded" ? 1 : null,
  };
}
function deferred<T>() {
  let finish!: (value: T) => void;
  const promise = new Promise<T>((resolve) => {
    finish = resolve;
  });
  return { promise, resolve: finish };
}
const scopes: ReturnType<typeof effectScope>[] = [];
function setup() {
  const scope = effectScope();
  scopes.push(scope);
  return scope.run(useSourceSeparationCandidate)!;
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.getSpeechSourceSeparation.mockResolvedValue(candidate);
  mocks.observeResourceRun.mockReturnValue(vi.fn<() => undefined>());
});
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

it("restores a completed candidate and stays busy until its audio metadata arrives", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("succeeded"));
  const loading = deferred<SpeechSourceSeparationCandidate>();
  mocks.getSpeechSourceSeparation.mockReturnValue(loading.promise);
  const separation = setup();
  const restoring = separation.restore("space", source);
  await vi.waitFor(() => expect(mocks.getSpeechSourceSeparation).toHaveBeenCalledWith("run"));
  expect(separation.busy.value).toBe(true);
  await separation.start("space", source);
  expect(mocks.startSpeechSourceSeparation).not.toHaveBeenCalled();
  loading.resolve(candidate);
  await restoring;
  expect(separation.candidate.value).toEqual(candidate);
  expect(separation.busy.value).toBe(false);
});

it("rejoins a running task rather than starting a second separation", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
  const separation = setup();
  await separation.restore("space", source);
  expect(separation.activeRunId.value).toBe("run");
  await separation.start("space", source);
  expect(mocks.startSpeechSourceSeparation).not.toHaveBeenCalled();
  expect(mocks.observeResourceRun).toHaveBeenCalledOnce();
});

it("does not apply a late restored task after the selected source changes", async () => {
  const loading = deferred<RunObservation | null>();
  mocks.getLatestSourceSeparationRun.mockReturnValueOnce(loading.promise).mockResolvedValueOnce(null);
  const separation = setup();
  const old = separation.restore("space", source);
  await separation.restore("other", { ...source, sourceRevision: "sha256:changed" });
  loading.resolve(run("succeeded"));
  await old;
  expect(separation.candidate.value).toBeUndefined();
  expect(mocks.getSpeechSourceSeparation).not.toHaveBeenCalled();
});

it("does not overwrite a new source with a late review result", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValueOnce(run("succeeded")).mockResolvedValueOnce(null);
  const separation = setup();
  await separation.restore("space", source);
  const loading = deferred<SpeechSourceSeparationCandidate>();
  mocks.reviewSpeechSourceSeparation.mockReturnValue(loading.promise);
  const reviewing = separation.review(true);
  await separation.restore("space", { ...source, sourceRevision: "sha256:changed" });
  loading.resolve({ ...candidate, reviewStatus: "accepted" });
  await reviewing;
  expect(separation.candidate.value).toBeUndefined();
  expect(separation.busy.value).toBe(false);
});

it.each(["restore", "start"] as const)(
  "keeps %s busy when a running task completes during subscription",
  async (operation) => {
    mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
    mocks.startSpeechSourceSeparation.mockResolvedValue(run("running"));
    const loading = deferred<SpeechSourceSeparationCandidate>();
    mocks.getSpeechSourceSeparation.mockReturnValue(loading.promise);
    mocks.observeResourceRun.mockImplementation((_run, _title, onChange) => {
      onChange?.(run("succeeded"));
      return vi.fn<() => undefined>();
    });
    const separation = setup();
    await separation[operation]("space", source);
    expect(separation.activeRunId.value).toBeUndefined();
    expect(separation.busy.value).toBe(true);
    const submissions = mocks.startSpeechSourceSeparation.mock.calls.length;
    await separation.start("space", source);
    expect(mocks.startSpeechSourceSeparation).toHaveBeenCalledTimes(submissions);
    loading.resolve(candidate);
    await vi.waitFor(() => expect(separation.busy.value).toBe(false));
    expect(separation.candidate.value).toEqual(candidate);
  },
);

it("treats a cancelled separation as a settled operation that can be started again", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("cancelled"));
  mocks.startSpeechSourceSeparation.mockResolvedValue(run("running"));
  const separation = setup();
  await separation.restore("space", source);
  expect(separation.error.value).toBeUndefined();
  expect(separation.busy.value).toBe(false);
  await separation.start("space", source);
  expect(mocks.startSpeechSourceSeparation).toHaveBeenCalledOnce();
});

it("submits one cancellation request and keeps a rejection on the active source", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
  const cancellation = deferred<void>();
  mocks.cancelResourceRun.mockReturnValue(cancellation.promise);
  const separation = setup();
  await separation.restore("space", source);
  const first = separation.cancel();
  const second = separation.cancel();
  expect(separation.busy.value).toBe(true);
  expect(mocks.cancelResourceRun).toHaveBeenCalledExactlyOnceWith("run");
  cancellation.resolve();
  await Promise.all([first, second]);
  expect(separation.busy.value).toBe(false);

  mocks.cancelResourceRun.mockRejectedValueOnce(new Error("Cancellation denied"));
  await separation.cancel();
  expect(separation.error.value).toBe("Cancellation denied");
  expect(separation.activeRunId.value).toBe("run");
});
