import type { Pinia } from "pinia";

import { useCreationLibraryStore } from "../foundation/ai/creation-library-store";
import { flushAiSettingsMutations } from "../foundation/ai/settings-mutations";
import { registerWindowCloseCommit } from "../foundation/lifecycle/window-close-coordinator";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";

const installed = new WeakSet<Pinia>();

export function installSettingsPersistence(pinia: Pinia): void {
  if (installed.has(pinia)) return;
  const creationStore = useCreationLibraryStore(pinia);
  const preferences = usePreferencesStore(pinia);
  registerWindowCloseCommit(async () => {
    const outcomes = await Promise.allSettled([
      flushAiSettingsMutations(pinia),
      creationStore.flush(),
      preferences.flush(),
    ]);
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
