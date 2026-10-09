import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";
import type * as vueI18n from "vue-i18n";

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
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof vueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const running: RunObservation = {
  id: "voice-run",
  scope: { kind: "application" },
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

function setup() {
  let candidates!: ReturnType<typeof useVoiceCandidates>;
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Harness = defineComponent({
    setup() {
      candidates = useVoiceCandidates();
      return () => null;
    },
  });
  const wrapper = mount(Harness, { global: { plugins: [[VueQueryPlugin, { queryClient }]] } });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return { candidates, wrapper };
}

beforeEach(() => {
  vi.resetAllMocks();
  mocks.getActiveVoiceCandidateRun.mockResolvedValue(running);
  mocks.listVoiceCandidates.mockResolvedValue([]);
  mocks.observeResourceRun.mockReturnValue(vi.fn());
});

it("restores the application candidate run independently of a Space", async () => {
  const { candidates } = setup();
  await flushPromises();
  expect(mocks.getActiveVoiceCandidateRun).toHaveBeenCalledOnce();
  expect(mocks.listVoiceCandidates).toHaveBeenCalledOnce();
  expect(candidates.activeRunId.value).toBe(running.id);
});

it("settles normal cancellation without presenting a failed voice proposal", async () => {
  const { candidates } = setup();
  await flushPromises();
  mocks.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state: "cancelled" });
  expect(candidates.activeRunId.value).toBeUndefined();
  expect(candidates.busy.value).toBe(false);
  expect(candidates.errors.value).toEqual([]);
  expect(mocks.getVoiceCandidate).not.toHaveBeenCalled();
});

it.each(["failed", "completed_with_errors"] as const)("retains %s as a visible failure", async (state) => {
  const { candidates } = setup();
  await flushPromises();
  mocks.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state });
  expect(candidates.busy.value).toBe(false);
  expect(candidates.errors.value).not.toEqual([]);
});

it("keeps a cancellation failure attached to the active operation", async () => {
  const { candidates } = setup();
  await flushPromises();
  mocks.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
  await candidates.cancel();
  expect(candidates.errors.value).toContain("Cancellation denied");
  expect(candidates.activeRunId.value).toBe(running.id);
});

it("detaches its view observer without cancelling the durable run", async () => {
  const stop = vi.fn<() => void>();
  mocks.observeResourceRun.mockReturnValue(stop);
  const { wrapper } = setup();
  await flushPromises();
  wrapper.unmount();
  expect(stop).toHaveBeenCalledOnce();
  expect(mocks.cancelResourceRun).not.toHaveBeenCalled();
});
