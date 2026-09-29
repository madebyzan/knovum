import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createIntegrationState } from "../integration-state";

describe("integration state persistence", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("does not save a revision again after an explicit flush", async () => {
    const save = vi.fn<(_value: { count: number }) => Promise<void>>().mockResolvedValue(undefined);
    const state = createIntegrationState({
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
    const state = createIntegrationState({
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
});
