import { defineStore } from "pinia";

import { loadAiModelSettings, resetAiModelSettings, saveAiModelSettings } from "./integration-settings";
import { createIntegrationState } from "./integration-state";
import { defaultAiModelSettings, type AiModelSettings } from "./schema/model-settings";

export const useAiSettingsStore = defineStore("ai-settings", () => {
  const { value: settings, ...state } = createIntegrationState<AiModelSettings>({
    initial: defaultAiModelSettings,
    load: loadAiModelSettings,
    save: saveAiModelSettings,
    reset: resetAiModelSettings,
  });

  return { settings, ...state };
});
