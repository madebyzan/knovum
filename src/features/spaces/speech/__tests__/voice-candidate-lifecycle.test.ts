import { flushPromises } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type * as observation from "../../runs/resource-run-observation";
import type * as speechIpc from "../ipc";
import { useVoiceCandidates } from "../use-voice-candidates";
import type * as ipc from "../voice-candidate-ipc";

const mocks = vi.hoisted(() => ({
  draftVoiceCandidate: vi.fn<typeof ipc.draftVoiceCandidate>(),
  cancelSpeechGeneration: vi.fn<typeof speechIpc.cancelSpeechGeneration>(),
  getActiveVoiceCandidateRun: vi.fn<typeof ipc.getActiveVoiceCandidateRun>(),
  listVoiceCandidates: vi.fn<typeof ipc.listVoiceCandidates>(),
  getVoiceCandidate: vi.fn<typeof ipc.getVoiceCandidate>(),
  observeResourceRun: vi.fn<typeof observation.observeResourceRun>(),
  cancelResourceRun: vi.fn<typeof observation.cancelResourceRun>(),
}));
vi.mock("../voice-candidate-ipc", () => mocks);
vi.mock("../ipc", () => mocks);
vi.mock("../../runs/resource-run-observation", () => mocks);
vi.mock("../../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));

const running: RunObservation = {
  id: "voice-run",
  homeSpaceId: "first",
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
const scopes: ReturnType<typeof effectScope>[] = [];
async function setup() {
  const owner = shallowRef("first");
  const scope = effectScope();
  scopes.push(scope);
  const candidates = scope.run(() => useVoiceCandidates(() => owner.value, vi.fn()))!;
  await flushPromises();
  return { candidates, owner, scope };
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.getActiveVoiceCandidateRun.mockResolvedValueOnce(running).mockResolvedValue(null);
  mocks.listVoiceCandidates.mockResolvedValue([]);
  mocks.observeResourceRun.mockReturnValue(vi.fn<() => undefined>(() => undefined));
});
afterEach(() => scopes.splice(0).forEach((scope) => scope.stop()));

it("settles normal cancellation without presenting a failed voice proposal", async () => {
  const { candidates } = await setup();
  mocks.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state: "cancelled" });
  expect(candidates.activeRunId.value).toBeUndefined();
  expect(candidates.busy.value).toBe(false);
  expect(candidates.error.value).toBeUndefined();
  expect(mocks.getVoiceCandidate).not.toHaveBeenCalled();
});

it.each(["failed", "completed_with_errors"] as const)("retains %s as a visible failure", async (state) => {
  const { candidates } = await setup();
  mocks.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state });
  expect(candidates.busy.value).toBe(false);
  expect(candidates.error.value).toBeTruthy();
});

it("keeps a real cancellation command failure attached to its active operation", async () => {
  const { candidates } = await setup();
  mocks.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
  await candidates.cancel();
  expect(candidates.error.value).toBe("Cancellation denied");
  expect(candidates.activeRunId.value).toBe(running.id);
});

it.each(["space-change", "dispose"] as const)(
  "ignores a late cancellation failure after %s",
  async (action) => {
    const { candidates, owner, scope } = await setup();
    let failCancellation!: (cause: unknown) => void;
    mocks.cancelResourceRun.mockReturnValue(
      new Promise((_resolve, reject) => {
        failCancellation = reject;
      }),
    );
    const pending = candidates.cancel();
    expect(mocks.cancelResourceRun).toHaveBeenCalledExactlyOnceWith(running.id);
    if (action === "dispose") scope.stop();
    else owner.value = "second";
    await flushPromises();
    failCancellation(new Error("Old task failure"));
    await pending;
    expect(candidates.error.value).toBeUndefined();
    expect(candidates.activeRunId.value).toBeUndefined();
  },
);
