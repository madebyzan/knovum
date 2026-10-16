import type { Pinia } from "pinia";

import { flushAiSettingsMutations } from "../foundation/ai/settings-mutations";
import { registerWindowCloseCommit } from "../foundation/lifecycle/window-close-coordinator";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";

const installed = new WeakSet<Pinia>();

export function installSettingsPersistence(pinia: Pinia): void {
  if (installed.has(pinia)) return;
  const preferences = usePreferencesStore(pinia);
  registerWindowCloseCommit(async () => {
    const outcomes = await Promise.allSettled([flushAiSettingsMutations(pinia), preferences.flush()]);
    const failures = outcomes.filter((outcome) => outcome.status === "rejected");
    if (failures.length) {
      throw new AggregateError(
        failures.map((failure) => failure.reason),
        "Application settings could not be saved",
      );
    }
  });
  installed.add(pinia);
}
