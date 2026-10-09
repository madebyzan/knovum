import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { defineComponent, shallowRef } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type { SavedCapturedWorkflow } from "../../contracts/generated/SavedCapturedWorkflow";
import { useCapturedWorkflowLibrary } from "../use-captured-workflow-library";
import { useCapturedWorkflowRun } from "../use-captured-workflow-run";

const ipc = vi.hoisted(() => ({
  start: vi.fn<() => Promise<RunObservation>>(),
  retry: vi.fn<() => Promise<RunObservation>>(),
  outputs: vi.fn<() => Promise<[]>>(),
  observe: vi.fn<() => () => void>(),
}));
vi.mock("../captured-workflow-ipc", () => ({
  startCapturedWorkflow: ipc.start,
  retryCapturedWorkflowRun: ipc.retry,
  listCapturedRunOutputs: ipc.outputs,
  listCapturedWorkflows: async () => ({ items: [] }),
  listCapturedWorkflowRuns: async () => [],
}));
vi.mock("../../runs/resource-run-observation", () => ({
  observeResourceRun: ipc.observe,
  cancelResourceRun: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => ({}) }));
const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => {
  for (const wrapper of mounted.splice(0)) wrapper.unmount();
});
beforeEach(() => {
  vi.clearAllMocks();
  ipc.observe.mockReturnValue(vi.fn<() => void>());
  ipc.outputs.mockResolvedValue([]);
});

const workflow: SavedCapturedWorkflow = {
  schemaVersion: 1,
  workflowId: "workflow",
  version: 1,
  name: "Workflow",
  candidate: { inputSlots: [], steps: [], outputSlots: [] },
  sourceSpaceId: "space",
  sourceDigest: "digest",
  selectedNodeIds: [],
  createdAt: 1,
};
const observation: RunObservation = {
  id: "run",
  scope: { kind: "space", spaceId: "space" },
  parentRunId: null,
  retryOfRunId: null,
  forkOfRunId: null,
  kind: "workflow",
  state: "running",
  stateVersion: 1,
  waitReason: null,
  inputTokens: 0,
  outputTokens: 0,
  totalTokens: 0,
  stepAttempts: 0,
  expandedSteps: 0,
  outputBytes: 0,
  createdAt: 1,
  updatedAt: 1,
  completedAt: null,
};
function runSession() {
  let session!: ReturnType<typeof useCapturedWorkflowRun>;
  mounted.push(
    mount(
      defineComponent({
        setup() {
          session = useCapturedWorkflowRun();
          return () => null;
        },
      }),
    ),
  );
  return session;
}
function startInput(isCurrent = () => true) {
  return { workflow, spaceId: "space", bindings: {}, requestId: "request", isCurrent };
}

it("returns a started run for history even if its UI request is no longer current", async () => {
  ipc.start.mockResolvedValueOnce(observation);
  const session = runSession();
  expect(await session.start(startInput(() => false))).toEqual(observation);
  expect(session.runId.value).toBe("");
  expect(ipc.observe).toHaveBeenCalledOnce();
});

it("returns a retried run and attaches it to the active session", async () => {
  ipc.start.mockResolvedValueOnce(observation);
  const session = runSession();
  await session.start(startInput());
  const retry = { ...observation, id: "retried", retryOfRunId: observation.id };
  ipc.retry.mockResolvedValueOnce(retry);
  expect(await session.retry(workflow.name)).toEqual(retry);
  expect(session.runId.value).toBe("retried");
});

it("exposes a failed start and does not return a successful observation", async () => {
  ipc.start.mockRejectedValueOnce(new Error("Admission denied"));
  const session = runSession();
  expect(await session.start(startInput())).toBeUndefined();
  expect(session.error.value).toBe("Admission denied");
  expect(session.submitting.value).toBe(false);
});

it("does not expose a retry failure after the active session is reset", async () => {
  ipc.start.mockResolvedValueOnce(observation);
  const session = runSession();
  await session.start(startInput());
  let rejectRetry!: (cause: unknown) => void;
  ipc.retry.mockReturnValueOnce(
    new Promise<RunObservation>((_resolve, reject) => {
      rejectRetry = reject;
    }),
  );
  const retrying = session.retry(workflow.name);
  session.reset();
  rejectRetry(new Error("Old retry failed"));
  expect(await retrying).toBeUndefined();
  expect(session.error.value).toBe("");
});

it("records a returned run in its origin space after the active space changes", async () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const spaceId = shallowRef("space");
  let library!: ReturnType<typeof useCapturedWorkflowLibrary>;
  mounted.push(
    mount(
      defineComponent({
        setup() {
          library = useCapturedWorkflowLibrary(spaceId);
          return () => null;
        },
      }),
      { global: { plugins: [[VueQueryPlugin, { queryClient }]] } },
    ),
  );
  await flushPromises();
  spaceId.value = "other-space";
  await flushPromises();
  const entry = { run: observation, workflowId: workflow.workflowId };
  library.addRun("space", entry);
  expect(queryClient.getQueryData(["spaces", "space", "captured-workflow-runs"])).toEqual([entry]);
  expect(library.history.value).toEqual([]);
  queryClient.clear();
});
