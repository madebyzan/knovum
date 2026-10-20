import { expect, it, vi } from "vitest";
import { effectScope } from "vue";

import type { SettingsLeaveParticipant } from "../../../shared/contracts/settings-leave";
import { useEditorLeave } from "../use-editor-leave";

const coordinator = vi.hoisted(() => ({
  register: vi.fn<(participant: SettingsLeaveParticipant) => () => void>(),
}));
vi.mock("../use-settings-leave-context", () => ({ useSettingsLeaveContext: () => coordinator }));

function editor(options: Partial<Parameters<typeof useEditorLeave>[0]> = {}) {
  const unregister = vi.fn<() => void>();
  coordinator.register.mockReturnValue(unregister);
  const scope = effectScope();
  const close = vi.fn<() => void>();
  const confirmDiscard = vi.fn<() => Promise<boolean>>(async () => true);
  const requestClose = scope.run(() =>
    useEditorLeave({
      pending: () => undefined,
      dirty: () => false,
      confirmDiscard,
      close,
      ...options,
    }),
  )!;
  const participant = coordinator.register.mock.lastCall![0] as SettingsLeaveParticipant;
  return { scope, unregister, close, confirmDiscard, requestClose, participant };
}

it("waits for the admitted write before checking whether changes remain", async () => {
  let finish!: (value: boolean) => void;
  let dirty = true;
  const pending = new Promise<boolean>((resolve) => {
    finish = resolve;
  });
  const state = editor({ pending: () => pending, dirty: () => dirty });
  const closing = state.requestClose();
  expect(state.close).not.toHaveBeenCalled();
  expect(state.confirmDiscard).not.toHaveBeenCalled();
  dirty = false;
  finish(true);
  expect(await closing).toBe(true);
  expect(state.confirmDiscard).not.toHaveBeenCalled();
  expect(state.close).toHaveBeenCalledOnce();
  state.scope.stop();
});

it("a failed write blocks close and navigation without asking to discard", async () => {
  const state = editor({ pending: () => Promise.resolve(false), dirty: () => true });
  expect(await state.requestClose()).toBe(false);
  expect(await state.participant.prepare()).toBe(false);
  expect(state.close).not.toHaveBeenCalled();
  expect(state.confirmDiscard).not.toHaveBeenCalled();
  state.scope.stop();
});

it("discard cancellation keeps the editor open", async () => {
  const state = editor({ dirty: () => true, confirmDiscard: async () => false });
  expect(await state.requestClose()).toBe(false);
  expect(state.close).not.toHaveBeenCalled();
  state.scope.stop();
});

it("navigation prepares without closing, then finalizes and unregisters on disposal", async () => {
  const state = editor({ dirty: () => true });
  expect(await state.participant.prepare()).toBe(true);
  expect(state.confirmDiscard).toHaveBeenCalledOnce();
  expect(state.close).not.toHaveBeenCalled();
  state.participant.finalize?.();
  expect(state.close).toHaveBeenCalledOnce();
  state.scope.stop();
  expect(state.unregister).toHaveBeenCalledOnce();
});
