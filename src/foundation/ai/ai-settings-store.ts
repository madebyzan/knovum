import { defineStore } from "pinia";

import {
  defaultAiModelSettings,
  loadAiModelSettings,
  resetAiModelSettings,
  saveAiModelSettings,
  type AiModelSettings,
} from "./integration-settings";
import { createIntegrationState } from "./integration-state";

export const useAiSettingsStore = defineStore("ai-settings", () => {
  const { value: settings, ...state } = createIntegrationState<AiModelSettings>({
    initial: defaultAiModelSettings,
    load: loadAiModelSettings,
    save: saveAiModelSettings,
    reset: resetAiModelSettings,
  });

  return { settings, ...state };
});
