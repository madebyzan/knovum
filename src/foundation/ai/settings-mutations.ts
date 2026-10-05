import pLimit from "p-limit";
import type { Pinia } from "pinia";

import { isApplicationTransitioning } from "../lifecycle/application-transition";
import { useAiSettingsStore } from "./ai-settings-store";

const serialize = pLimit(1);

export function mutateAiSettings<T>(mutation: () => Promise<T>): Promise<T> {
  if (isApplicationTransitioning()) return Promise.reject(new Error("AI settings are transitioning"));
  return serialize(mutation);
}

export function flushAiSettingsMutations(pinia?: Pinia): Promise<void> {
  return serialize(() => useAiSettingsStore(pinia).flush());
}

export function resetAiSettings(): Promise<void> {
  return mutateAiSettings(() => useAiSettingsStore().reset());
}
