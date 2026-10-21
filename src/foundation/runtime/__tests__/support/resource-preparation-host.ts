import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, vi } from "vitest";

import type * as localResources from "../../local-resources";

/** The resource engine a preparation task drives; suites route their `vi.mock` factories here. */
export const resourceEngine = {
  prepareResources: vi.fn<typeof localResources.prepareResources>(),
  cancelResourcePreparation: vi
    .fn<typeof localResources.cancelResourcePreparation>()
    .mockResolvedValue(undefined),
  listResourcePreparations: vi.fn<typeof localResources.listResourcePreparations>().mockResolvedValue([]),
};
export const resourceQuery = {
  invalidateLocalResources: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
};
export const logPlugin = { error: vi.fn<(message: string) => Promise<void>>() };
export const keyedI18n = { i18n: { global: { t: (key: string) => key } } };

/** Each test starts with a fresh task store on fake timers. */
export function useTaskLifetime() {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
    vi.clearAllMocks();
  });
  afterEach(() => vi.useRealTimers());
}
