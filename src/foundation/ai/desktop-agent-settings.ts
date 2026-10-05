import { cloneDeep } from "es-toolkit";

import type { AgentKind } from "../../shared/contracts/generated/AgentKind";
import { useAiSettingsStore } from "./ai-settings-store";
import { mutateAiSettings } from "./settings-mutations";

export function setDesktopAgentEnabled(kind: AgentKind, enabled: boolean): Promise<void> {
  return mutateAiSettings(async () => {
    const store = useAiSettingsStore();
    const previousAgents = cloneDeep(store.settings.desktopAgents);
    const previousDefaults = cloneDeep(store.settings.defaults);
    const index = store.settings.desktopAgents.findIndex((agent) => agent.kind === kind);
    if (enabled && index === -1) store.settings.desktopAgents.push({ kind, enabled: true });
    else if (!enabled && index >= 0) {
      store.settings.desktopAgents.splice(index, 1);
      for (const purpose of Object.keys(store.settings.defaults) as Array<
        keyof typeof store.settings.defaults
      >) {
        const target = store.settings.defaults[purpose];
        if (target?.type === "agent" && target.runtimeKind === kind) store.settings.defaults[purpose] = null;
      }
    }
    try {
      await store.flush();
    } catch (error) {
      store.settings.desktopAgents = previousAgents;
      store.settings.defaults = previousDefaults;
      throw error;
    }
  });
}
