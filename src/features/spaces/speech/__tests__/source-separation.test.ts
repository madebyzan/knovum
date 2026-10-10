import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope } from "vue";
import type * as vueI18n from "vue-i18n";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type * as runObservation from "../../runs/resource-run-observation";
import type * as sourceSeparationIpc from "../source-separation-ipc";
import { useSourceSeparation } from "../use-source-separation";

const mocks = vi.hoisted(() => ({
  getLatestSourceSeparationRun: vi.fn<typeof sourceSeparationIpc.getLatestSourceSeparationRun>(),
  startSpeechSourceSeparation: vi.fn<typeof sourceSeparationIpc.startSpeechSourceSeparation>(),
  observeResourceRun: vi.fn<typeof runObservation.observeResourceRun>(),
  cancelResourceRun: vi.fn<typeof runObservation.cancelResourceRun>(),
}));
vi.mock("../source-separation-ipc", () => mocks);
vi.mock("../../runs/resource-run-observation", () => mocks);
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof vueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
const source = { assetPath: "/media.wav", sourceRevision: "sha256:source", durationMs: 1_000 };
function run(state: RunObservation["state"]): RunObservation {
  return {
    id: "run",
    scope: { kind: "space", spaceId: "space" },
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
  return scope.run(useSourceSeparation)!;
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.observeResourceRun.mockReturnValue(vi.fn<() => undefined>());
});
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

it("settles a completed derivation without a result review or insertion stage", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("succeeded"));
  const separation = setup();
  await separation.restore("space", "source", source);
  expect(separation.activeRunId.value).toBeUndefined();
  expect(separation.isBusy.value).toBe(false);
  expect(separation.error.value).toBeUndefined();
  expect(mocks.observeResourceRun).not.toHaveBeenCalled();
});

it("rejoins the selected node's running task and prevents duplicate submission", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
  const separation = setup();
  await separation.restore("space", "source", source);
  await separation.start("space", "source", source);
  expect(separation.activeRunId.value).toBe("run");
  expect(mocks.getLatestSourceSeparationRun).toHaveBeenCalledWith("space", "source", source);
  expect(mocks.startSpeechSourceSeparation).not.toHaveBeenCalled();
});

it("ignores a late restored task after the selected node changes", async () => {
  const loading = deferred<RunObservation | null>();
  mocks.getLatestSourceSeparationRun.mockReturnValueOnce(loading.promise).mockResolvedValueOnce(null);
  const separation = setup();
  const old = separation.restore("space", "source", source);
  await separation.restore("space", "other-node", source);
  loading.resolve(run("running"));
  await old;
  expect(separation.activeRunId.value).toBeUndefined();
  expect(mocks.observeResourceRun).not.toHaveBeenCalled();
});

it("keeps the submitted derivation in task observation after its node UI closes", async () => {
  const submitted = deferred<RunObservation>();
  mocks.startSpeechSourceSeparation.mockReturnValue(submitted.promise);
  const separation = setup();
  const starting = separation.start("space", "source", source);
  await separation.start("space", "source", source);
  expect(mocks.startSpeechSourceSeparation).toHaveBeenCalledExactlyOnceWith({
    requestId: expect.any(String),
    spaceId: "space",
    sourceNodeId: "source",
    source,
    vocalsLabel: "spaces.speech.separation.stems.vocals",
    backgroundLabel: "spaces.speech.separation.stems.background",
  });
  separation.reset();
  submitted.resolve(run("running"));
  await starting;
  expect(separation.activeRunId.value).toBeUndefined();
  expect(mocks.observeResourceRun).toHaveBeenCalledWith(run("running"), "spaces.speech.separation.task");
});

it.each(["restore", "start"] as const)(
  "settles %s when publication completes during subscription",
  async (operation) => {
    mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
    mocks.startSpeechSourceSeparation.mockResolvedValue(run("running"));
    mocks.observeResourceRun.mockImplementation((_run, _title, onChange) => {
      onChange?.(run("succeeded"));
      return vi.fn<() => undefined>();
    });
    const separation = setup();
    await separation[operation]("space", "source", source);
    expect(separation.activeRunId.value).toBeUndefined();
    expect(separation.isBusy.value).toBe(false);
    expect(separation.error.value).toBeUndefined();
  },
);

it("allows a cancelled derivation to be started again", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("cancelled"));
  mocks.startSpeechSourceSeparation.mockResolvedValue(run("running"));
  const separation = setup();
  await separation.restore("space", "source", source);
  expect(separation.error.value).toBeUndefined();
  await separation.start("space", "source", source);
  expect(separation.activeRunId.value).toBe("run");
});

it("refreshes a derivation that finishes before the start reply reaches its node UI", async () => {
  mocks.startSpeechSourceSeparation.mockResolvedValue(run("succeeded"));
  const separation = setup();
  await separation.start("space", "source", source);
  expect(mocks.observeResourceRun).toHaveBeenCalledWith(run("succeeded"), "spaces.speech.separation.task");
  expect(separation.activeRunId.value).toBeUndefined();
  expect(separation.isBusy.value).toBe(false);
  expect(separation.error.value).toBeUndefined();
});

it("submits one cancellation and reports a rejected cancellation on the active source", async () => {
  mocks.getLatestSourceSeparationRun.mockResolvedValue(run("running"));
  const cancellation = deferred<void>();
  mocks.cancelResourceRun.mockReturnValue(cancellation.promise);
  const separation = setup();
  await separation.restore("space", "source", source);
  const first = separation.cancel();
  const second = separation.cancel();
  expect(separation.isBusy.value).toBe(true);
  expect(mocks.cancelResourceRun).toHaveBeenCalledExactlyOnceWith("run");
  cancellation.resolve();
  await Promise.all([first, second]);
  mocks.cancelResourceRun.mockRejectedValueOnce(new Error("Cancellation denied"));
  await separation.cancel();
  expect(separation.error.value).toBe("Cancellation denied");
  expect(separation.activeRunId.value).toBe("run");
});
