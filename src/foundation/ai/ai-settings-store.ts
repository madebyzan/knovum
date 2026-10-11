import { defineStore } from "pinia";

import { loadAiModelSettings, resetAiModelSettings, saveAiModelSettings } from "./integration-settings";
import { DEFAULT_AI_MODEL_SETTINGS, type AiModelSettings } from "./schema/model-settings";
import { useIntegrationSettingsState } from "./use-integration-settings-state";

export const useAiSettingsStore = defineStore("ai-settings", () => {
  const { value: settings, ...state } = useIntegrationSettingsState<AiModelSettings>({
    initial: DEFAULT_AI_MODEL_SETTINGS,
    load: loadAiModelSettings,
    save: saveAiModelSettings,
    reset: resetAiModelSettings,
  });

  return { settings, ...state };
});
