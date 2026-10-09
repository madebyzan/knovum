import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { useIntegrationSettingsState } from "../use-integration-settings-state";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

function session() {
  const save = vi.fn<(_value: { count: number }) => Promise<void>>().mockResolvedValue(undefined);
  const scope = effectScope();
  const state = scope.run(() =>
    useIntegrationSettingsState({
      initial: { count: 0 },
      load: async () => ({ count: 0 }),
      save,
      reset: async () => {},
    }),
  )!;
  return { scope, state, save };
}

describe("integration settings scope", () => {
  it("stops observing edits after its owner is disposed", async () => {
    const { scope, state, save } = session();
    await state.ensureLoaded();
    state.value.value.count = 1;
    await state.flush();
    scope.stop();
    state.value.value.count = 2;
    await vi.advanceTimersByTimeAsync(1200);
    expect(save).toHaveBeenCalledOnce();
    expect(save).toHaveBeenLastCalledWith({ count: 1 });
  });

  it("cancels its timer while preserving dirty edits for an explicit flush", async () => {
    const { scope, state, save } = session();
    await state.ensureLoaded();
    state.value.value.count = 1;
    scope.stop();
    await vi.advanceTimersByTimeAsync(1200);
    expect(save).not.toHaveBeenCalled();
    await state.flush();
    expect(save).toHaveBeenCalledWith({ count: 1 });
  });
});
