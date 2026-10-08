import type { EventCallback, listen } from "@tauri-apps/api/event";
import { beforeEach, expect, it, vi } from "vitest";

import type { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { DictationSnapshot } from "../dictation-contracts";
import { useDictationSession } from "../use-dictation-session";

const mocks = vi.hoisted(() => ({
  listen: vi.fn<typeof listen<DictationSnapshot>>(),
  invoke: vi.fn<typeof invokeCommand>(),
  release: vi.fn<() => void>(),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

const snapshot: DictationSnapshot = {
  phase: "idle",
  resources: null,
  sessionId: null,
  startedAt: null,
  targetApplication: null,
  text: null,
  language: null,
  insertion: null,
  errorCode: null,
  errorMessage: null,
};
beforeEach(() => vi.clearAllMocks());

it("keeps a newer event when the initial state request resolves late", async () => {
  let received: EventCallback<DictationSnapshot> | undefined;
  mocks.listen.mockImplementation((_name: string, callback: typeof received) => {
    received = callback;
    return Promise.resolve(mocks.release);
  });
  let finish: ((value: DictationSnapshot) => void) | undefined;
  mocks.invoke.mockReturnValue(
    new Promise<DictationSnapshot>((resolve) => {
      finish = resolve;
    }),
  );
  const session = useDictationSession();
  const start = session.start();
  await Promise.resolve();
  received?.({
    event: "dictation-state",
    id: 1,
    payload: { ...snapshot, phase: "transcribing", sessionId: "current" },
  });
  finish?.(snapshot);
  await start;
  expect(session.state.value.sessionId).toBe("current");
  session.stop();
});

it("releases a subscription that finishes registering after disposal", async () => {
  let finish: ((value: () => void) => void) | undefined;
  mocks.listen.mockReturnValue(
    new Promise<() => void>((resolve) => {
      finish = resolve;
    }),
  );
  const session = useDictationSession();
  const start = session.start();
  session.stop();
  finish?.(mocks.release);
  await start;
  expect(mocks.release).toHaveBeenCalledOnce();
  expect(mocks.invoke).not.toHaveBeenCalled();
});

it("cleans up the listener if initial state retrieval fails", async () => {
  mocks.listen.mockResolvedValue(mocks.release);
  mocks.invoke.mockRejectedValue(new Error("State is unavailable"));
  const session = useDictationSession();
  await expect(session.start()).rejects.toThrow("State is unavailable");
  expect(mocks.release).toHaveBeenCalledOnce();
  session.stop();
  expect(mocks.release).toHaveBeenCalledOnce();
});
