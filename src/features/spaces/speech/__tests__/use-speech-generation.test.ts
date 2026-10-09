import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import { useSpeechGeneration } from "../use-speech-generation";

const mocks = vi.hoisted(() => ({
  activeSpaceId: "space",
  generateSpeechProject: vi.fn<() => Promise<RunObservation>>(),
  cancelSpeechGeneration: vi.fn<() => Promise<boolean>>(),
  cancelResourceRun: vi.fn<() => Promise<void>>(),
  observeResourceRun: vi.fn<(run: RunObservation, title: string) => void>(),
  attachRun: vi.fn<(run: RunObservation, projectId: string, spaceId: string) => void>(),
}));
vi.mock("../ipc", () => mocks);
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => mocks }));
vi.mock("../../runs/resource-run-observation", () => mocks);
vi.mock("../use-speech-preview", () => ({
  useSpeechPreview: () => ({ jobId: shallowRef(), reset: vi.fn<() => void>() }),
}));
vi.mock("../use-speech-run-observation", () => ({
  useSpeechRunObservation: () => ({ attachRun: mocks.attachRun, reset: vi.fn<() => void>() }),
}));

const run: RunObservation = {
  id: "run",
  scope: { kind: "space", spaceId: "space" },
  parentRunId: null,
  retryOfRunId: null,
  forkOfRunId: null,
  kind: "workflow",
  state: "queued",
  stateVersion: 0,
  waitReason: null,
  inputTokens: 0,
  outputTokens: 0,
  totalTokens: 0,
  stepAttempts: 0,
  expandedSteps: 1,
  outputBytes: 0,
  createdAt: 1,
  updatedAt: 1,
  completedAt: null,
};
const project: SpeechProjectSnapshot = {
  project: {
    id: "project",
    spaceId: "space",
    sourceNodeId: "node",
    projectType: "short_dubbing",
    title: "Speech",
    activeRevisionId: "revision",
    activeRevision: 1,
    segmentCount: 1,
    updatedAt: 1,
  },
  revision: {
    id: "revision",
    projectId: "project",
    revision: 1,
    sourceRevision: "source",
    pronunciationLexicon: {},
    outputConfig: { format: "wav", wavPrecision: "pcm16" },
    createdAt: 1,
  },
  segments: [],
};
const scopes: ReturnType<typeof effectScope>[] = [];
function setup() {
  const selected = shallowRef(project);
  const owner = {
    project: () => selected.value,
    reloadProject: vi.fn<(projectId: string) => Promise<void>>().mockResolvedValue(undefined),
    setError: vi.fn<(message?: string) => void>(),
  };
  const scope = effectScope();
  scopes.push(scope);
  const generation = scope.run(() => useSpeechGeneration(owner))!;
  let finish!: (value: RunObservation) => void;
  let fail!: (cause: unknown) => void;
  mocks.generateSpeechProject.mockReturnValue(
    new Promise((resolve, reject) => {
      finish = resolve;
      fail = reject;
    }),
  );
  return { generation, owner, selected, resolve: finish, reject: fail };
}
beforeEach(() => {
  vi.clearAllMocks();
  mocks.activeSpaceId = "space";
  mocks.cancelSpeechGeneration.mockResolvedValue(true);
});
afterEach(() => scopes.splice(0).forEach((scope) => scope.stop()));

it("uses the request identity to cancel while first-use resources are being prepared", async () => {
  const { generation, resolve } = setup();
  const pending = generation.generateProject();
  const requestId = generation.activeJobId.value;
  expect(requestId).toBeTypeOf("string");
  await generation.cancelGeneration();
  expect(mocks.cancelSpeechGeneration).toHaveBeenCalledExactlyOnceWith(requestId);
  expect(mocks.cancelResourceRun).not.toHaveBeenCalled();
  resolve(run);
  await pending;
  expect(generation.activeJobId.value).toBeUndefined();
});

it("keeps a late created Run in Task Center without attaching it to another node", async () => {
  const { generation, owner, resolve, selected } = setup();
  const pending = generation.generateProject();
  selected.value = { ...project, project: { ...project.project, id: "other" } };
  generation.reset();
  resolve(run);
  await pending;
  expect(mocks.observeResourceRun).toHaveBeenCalledWith(run, "Speech");
  expect(mocks.attachRun).not.toHaveBeenCalled();
  expect(owner.reloadProject).not.toHaveBeenCalled();
});

it("treats preparation cancellation as an expected user action", async () => {
  const { generation, owner, reject } = setup();
  const pending = generation.generateProject();
  reject(new Error("Cancelled", { cause: { code: "cancelled", message: "Cancelled" } }));
  await pending;
  expect(owner.setError).not.toHaveBeenCalled();
  expect(generation.submitting.value).toBe(false);
});
