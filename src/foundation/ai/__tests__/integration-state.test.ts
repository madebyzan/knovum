import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { useIntegrationSettingsState } from "../use-integration-settings-state";

const scopes: ReturnType<typeof effectScope>[] = [];
function integrationState(options: Parameters<typeof useIntegrationSettingsState<{ count: number }>>[0]) {
  const scope = effectScope();
  scopes.push(scope);
  return scope.run(() => useIntegrationSettingsState(options))!;
}

describe("integration state persistence", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => {
    for (const scope of scopes.splice(0)) scope.stop();
    vi.useRealTimers();
  });

  it("does not save a revision again after an explicit flush", async () => {
    const save = vi.fn<(_value: { count: number }) => Promise<void>>().mockResolvedValue(undefined);
    const state = integrationState({
      initial: { count: 0 },
      load: async () => ({ count: 0 }),
      save,
      reset: async () => {},
    });
    await state.ensureLoaded();

    state.value.value.count = 1;
    await state.flush();
    await vi.advanceTimersByTimeAsync(1200);

    expect(save).toHaveBeenCalledOnce();
    expect(save).toHaveBeenCalledWith({ count: 1 });
  });

  it("reports a failed explicit flush to its caller", async () => {
    const failure = new Error("save failed");
    const state = integrationState({
      initial: { count: 0 },
      load: async () => ({ count: 0 }),
      save: vi.fn<() => Promise<void>>().mockRejectedValue(failure),
      reset: async () => {},
    });
    await state.ensureLoaded();

    state.value.value.count = 1;
    await expect(state.flush()).rejects.toBe(failure);
    expect(state.saveError.value).toBe(failure);
  });

  it("settles an older save before resetting and persists the next edit", async () => {
    let release!: () => void;
    const save = vi.fn<(_value: { count: number }) => Promise<void>>().mockResolvedValue(undefined);
    save.mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          release = resolve;
        }),
    );
    const reset = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const state = integrationState({
      initial: { count: 0 },
      load: async () => ({ count: 0 }),
      save,
      reset,
    });
    await state.ensureLoaded();
    state.value.value.count = 1;
    const saving = state.flush();
    await vi.advanceTimersByTimeAsync(0);
    const resetting = state.reset();
    const closing = state.flush();
    expect(reset).not.toHaveBeenCalled();
    release();
    await Promise.all([saving, resetting, closing]);
    state.value.value.count = 2;
    await state.flush();
    expect(save).toHaveBeenLastCalledWith({ count: 2 });
    expect(reset).toHaveBeenCalledOnce();
  });

  it("clears failed load state after a successful reset", async () => {
    const state = integrationState({
      initial: { count: 0 },
      load: async () => {
        throw new Error("invalid settings");
      },
      save: async () => {},
      reset: async () => {},
    });
    await state.ensureLoaded();
    expect(state.loadError.value).toBeDefined();
    await state.reset();
    expect(state.ready.value).toBe(true);
    expect(state.loadError.value).toBeUndefined();
    expect(state.saveError.value).toBeUndefined();
  });

  it("clears a failed save after resetting and preserves the next saved revision", async () => {
    const save = vi
      .fn<(_value: { count: number }) => Promise<void>>()
      .mockRejectedValueOnce(new Error("write failed"))
      .mockResolvedValue(undefined);
    const state = integrationState({
      initial: { count: 0 },
      load: async () => ({ count: 0 }),
      save,
      reset: async () => {},
    });
    await state.ensureLoaded();
    state.value.value.count = 1;
    await expect(state.flush()).rejects.toThrow("write failed");
    await state.reset();
    expect(state.saveError.value).toBeUndefined();
    state.value.value.count = 2;
    await state.flush();
    expect(save).toHaveBeenLastCalledWith({ count: 2 });
  });

  it("retains a usable value and exposes reset failures", async () => {
    const failure = new Error("reset failed");
    const state = integrationState({
      initial: { count: 0 },
      load: async () => ({ count: 1 }),
      save: async () => {},
      reset: async () => {
        throw failure;
      },
    });
    await state.ensureLoaded();
    await expect(state.reset()).rejects.toBe(failure);
    expect(state.value.value.count).toBe(1);
    expect(state.loading.value).toBe(false);
    expect(state.loadError.value).toBe(failure);
  });

  it("reports a failed reload with unsaved edits instead of looping its save barrier", async () => {
    const failure = new Error("reload failed");
    const load = vi
      .fn<() => Promise<{ count: number }>>()
      .mockResolvedValueOnce({ count: 0 })
      .mockRejectedValueOnce(failure);
    const save = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const state = integrationState({ initial: { count: 0 }, load, save, reset: async () => {} });
    await state.ensureLoaded();
    state.value.value.count = 1;
    await state.reload();
    await expect(state.flush()).rejects.toBe(failure);
    expect(save).not.toHaveBeenCalled();
  });
});
