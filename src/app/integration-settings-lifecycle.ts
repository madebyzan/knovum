import type { Pinia } from "pinia";

import { useCreationLibraryStore } from "../foundation/ai/creation-library-store";
import { flushAiSettingsMutations } from "../foundation/ai/settings-mutations";
import { registerWindowCloseCommit } from "../foundation/lifecycle/window-close-coordinator";

const installed = new WeakSet<Pinia>();

export function installIntegrationSettingsPersistence(pinia: Pinia): void {
  if (installed.has(pinia)) return;
  const creationStore = useCreationLibraryStore(pinia);
  registerWindowCloseCommit(async () => {
    const outcomes = await Promise.allSettled([flushAiSettingsMutations(pinia), creationStore.flush()]);
    const failures = outcomes.filter((outcome) => outcome.status === "rejected");
    if (failures.length) {
      throw new AggregateError(
        failures.map((failure) => failure.reason),
        "Integration settings could not be saved",
      );
    }
  });
  installed.add(pinia);
}
