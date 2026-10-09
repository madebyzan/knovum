import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { ActiveConversationRun } from "../../contracts/generated/ActiveConversationRun";
import type { RunObservation } from "../../contracts/generated/RunObservation";
import { useConversationRunRestoration } from "../use-conversation-run-restoration";

const bridge = vi.hoisted(() => ({
  list: vi.fn<(spaceId: string) => Promise<ActiveConversationRun[]>>(),
  error: vi.fn<(runId: string) => Promise<string | null>>(),
  invoke: vi.fn<() => Promise<void>>(),
  listeners: new Set<(run: RunObservation) => void>(),
}));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: bridge.invoke }));
vi.mock("../conversation-ipc", () => ({
  listActiveConversationRuns: bridge.list,
  getConversationTaskError: bridge.error,
}));
vi.mock("../conversation-run-observation", () => ({
  observeConversationRun: ({ onUpdate }: { onUpdate: (run: RunObservation) => void }) => {
    bridge.listeners.add(onUpdate);
    return () => bridge.listeners.delete(onUpdate);
  },
}));

const run: RunObservation = {
  id: "run",
  scope: { kind: "space", spaceId: "space" },
  parentRunId: null,
  retryOfRunId: null,
  forkOfRunId: null,
  kind: "builtin_agent",
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
const active: ActiveConversationRun = {
  run,
  conversationId: "conversation",
  branchId: "branch",
  title: "Research",
};
const scopes: ReturnType<typeof effectScope>[] = [];
function session() {
  const state = {
    spaceId: shallowRef<string | undefined>("space"),
    selectedConversationId: shallowRef("conversation"),
    activeRun: shallowRef<RunObservation>(),
    activeRunId: shallowRef<string>(),
    activeConversationId: shallowRef(""),
    activeSpaceId: shallowRef<string>(),
    taskState: shallowRef<string>(),
    loading: shallowRef(false),
    error: shallowRef<string>(),
    activityRevision: shallowRef(0),
    refresh: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    select: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    onDetach: vi.fn<() => void>(),
  };
  const scope = effectScope();
  scopes.push(scope);
  return { scope, state, restoration: scope.run(() => useConversationRunRestoration(state))! };
}
beforeEach(() => {
  bridge.listeners.clear();
  bridge.list.mockReset().mockResolvedValue([active]);
  bridge.error.mockReset();
  bridge.invoke.mockReset();
});
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

describe("conversation restoration observation ownership", () => {
  it.each(["restored", "submitted"] as const)(
    "does not apply an older diagnostic to a newly %s run",
    async (kind) => {
      let release!: (reason: string | null) => void;
      const diagnostic = new Promise<string | null>((resolve) => (release = resolve));
      bridge.error.mockReturnValue(diagnostic);
      const { state, restoration } = session();
      await restoration.restoreForSpace("space");
      [...bridge.listeners][0]!({ ...run, state: "failed", completedAt: 2 });
      if (kind === "restored") {
        bridge.list.mockResolvedValue([{ ...active, run: { ...run, id: "next-run" } }]);
        await restoration.restoreForSpace("space");
      } else {
        state.activeRun.value = { ...run, id: "next-run" };
        state.activeRunId.value = "next-run";
      }
      release("previous run failure");
      await diagnostic;
      expect(state.activeRunId.value).toBe("next-run");
      expect(state.error.value).toBeUndefined();
    },
  );
  it.each(["resume", "retryModel"] as const)(
    "ignores a late %s failure after disposal",
    async (operation) => {
      let rejectCommand!: (cause: unknown) => void;
      bridge.invoke.mockReturnValue(new Promise<void>((_resolve, reject) => (rejectCommand = reject)));
      const { scope, state, restoration } = session();
      await restoration.restoreForSpace("space");
      state.taskState.value = "paused";
      state.activeRun.value = { ...run, state: "waiting" };
      const command = restoration[operation]();
      scope.stop();
      rejectCommand(new Error("obsolete command error"));
      await command;
      expect(state.error.value).toBeUndefined();
    },
  );
  it("detaches its listener when its scope stops and ignores already queued updates", async () => {
    const { scope, state, restoration } = session();
    await restoration.restoreForSpace("space");
    const listener = [...bridge.listeners][0]!;
    scope.stop();
    listener({ ...run, stateVersion: 2 });
    expect(bridge.listeners.size).toBe(0);
    expect(state.activityRevision.value).toBe(1);
    expect(state.activeRun.value?.stateVersion).toBe(1);
    expect(state.onDetach).toHaveBeenCalledOnce();
  });

  it("does not attach a run whose lookup finishes after disposal", async () => {
    let release!: (runs: ActiveConversationRun[]) => void;
    bridge.list.mockReturnValue(new Promise((resolve) => (release = resolve)));
    const { scope, state, restoration } = session();
    const restoring = restoration.restoreForSpace("space");
    scope.stop();
    release([active]);
    await restoring;
    expect(bridge.listeners.size).toBe(0);
    expect(state.activeRunId.value).toBeUndefined();
  });

  it("ignores a terminal diagnostic and selection refresh after disposal", async () => {
    let release!: (diagnostic: string | null) => void;
    bridge.error.mockReturnValue(new Promise((resolve) => (release = resolve)));
    let finishRefresh!: () => void;
    const { scope, state, restoration } = session();
    state.refresh.mockReturnValue(new Promise<void>((resolve) => (finishRefresh = resolve)));
    await restoration.restoreForSpace("space");
    [...bridge.listeners][0]!({ ...run, state: "failed", completedAt: 2 });
    scope.stop();
    release("obsolete failure");
    finishRefresh();
    await vi.waitFor(() => expect(bridge.listeners.size).toBe(0));
    expect(state.error.value).toBeUndefined();
    expect(state.select).not.toHaveBeenCalled();
  });
});
