export const EXTENSION_TABS = ["plugins", "apps", "mcp", "skills"] as const;
export type ExtensionTab = (typeof EXTENSION_TABS)[number];
export interface ExtensionDestination {
  tab: ExtensionTab;
  id?: string | undefined;
  connectorId?: string | undefined;
  action?: "create" | "browse" | undefined;
}
export function isExtensionTab(value: unknown): value is ExtensionTab {
  return EXTENSION_TABS.some((tab) => tab === value);
}
