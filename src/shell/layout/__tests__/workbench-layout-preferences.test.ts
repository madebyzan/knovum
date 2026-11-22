import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import { usePreferencesStore } from "../../../foundation/preferences/preferences-store";
import { preferenceDefaults } from "../../../shared/contracts/generated/contract-values";
import { useWorkbenchLayoutPreferences } from "../use-workbench-layout-preferences";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("../../../foundation/i18n/instance", () => ({
  i18n: { global: { locale: { value: "en" } } },
  systemLocale: () => "en",
}));
vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ get: async () => undefined, set: async () => {}, save: async () => {} }),
}));
vi.mock("@tauri-apps/plugin-os", () => ({ locale: async () => "en" }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    theme: async () => "light",
    setTheme: async () => {},
    onThemeChanged: async () => () => {},
  }),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: async () => {} }));

describe("useWorkbenchLayoutPreferences", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("restores sidebar and inspector visibility when startup restore is enabled", () => {
    const preferences = usePreferencesStore();
    preferences.restorePreviousWorkspace = true;
    preferences.workbench = {
      sidebarVisible: false,
      sidebarWidth: 260,
      rightSidebarVisible: true,
      rightSidebarWidth: 420,
    };

    const scope = effectScope();
    const layout = scope.run(() => useWorkbenchLayoutPreferences())!;
    expect(layout.sidebarPinned.value).toBe(false);
    expect(layout.sidebarWidth.value).toBe(260);
    expect(layout.rightSidebarOpen.value).toBe(true);
    expect(layout.rightSidebarWidth.value).toBe(420);
    scope.stop();
  });

  it("uses default sidebar and inspector visibility when startup restore is disabled", () => {
    const preferences = usePreferencesStore();
    preferences.restorePreviousWorkspace = false;
    preferences.workbench = {
      sidebarVisible: false,
      sidebarWidth: 260,
      rightSidebarVisible: true,
      rightSidebarWidth: 420,
    };

    const scope = effectScope();
    const layout = scope.run(() => useWorkbenchLayoutPreferences())!;
    expect(layout.sidebarPinned.value).toBe(preferenceDefaults.workbench.sidebarVisible);
    expect(layout.sidebarWidth.value).toBe(260);
    expect(layout.rightSidebarOpen.value).toBe(preferenceDefaults.workbench.rightSidebarVisible);
    expect(layout.rightSidebarWidth.value).toBe(420);
    scope.stop();
  });
});
