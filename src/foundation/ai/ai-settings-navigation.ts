/** The Settings panes that resolve an AI requirement. */
export type AiSettingsPane = "aiProviders" | "aiRouting" | "localModels" | "desktopAgents";

type AiSettingsNavigator = (pane: AiSettingsPane) => Promise<void>;

let navigator: AiSettingsNavigator | undefined;

/**
 * Settings is a route the composition root owns, so foundation code names the pane it needs and the
 * root decides how to get there. Installed once at bootstrap.
 */
export function installAiSettingsNavigation(navigate: AiSettingsNavigator) {
  navigator = navigate;
}

export function openAiSettings(pane: AiSettingsPane) {
  if (!navigator) throw new Error("AI settings navigation is not installed");
  return navigator(pane);
}
