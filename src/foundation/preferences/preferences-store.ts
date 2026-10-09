import { getCurrentWindow, type Theme } from "@tauri-apps/api/window";
import { locale as getOsLocale } from "@tauri-apps/plugin-os";
import { load } from "@tauri-apps/plugin-store";
import { isNil, pick } from "es-toolkit";
import pLimit from "p-limit";
import { defineStore } from "pinia";
import { reactive, toRefs } from "vue";

import { i18n, systemLocale, type AppLocale } from "../i18n/i18n";
import { invokeCommand } from "../ipc/invoke-command";
import { isApplicationTransitioning } from "../lifecycle/application-transition";
import {
  decodePreferences,
  defaultPreferences,
  PREFERENCES_SCHEMA_VERSION,
  type DensityPreference,
  type LocalePreference,
  type Preferences,
  type TextSizePreference,
  type ThemePreference,
  type WorkbenchPreferences,
} from "./schema/preferences";

const SETTINGS_FILE = "settings.json";
const SETTINGS_KEY = "preferences";
const currentWindow = getCurrentWindow();
let systemTheme: Theme = "light";
let osLocale: AppLocale = "en";
let stopThemeListener: (() => void) | undefined;

function snapshot(state: Preferences): Preferences {
  return {
    ...pick(state, ["theme", "density", "textSize", "locale", "backgroundTaskNotifications"]),
    workbench: { ...state.workbench },
  };
}

function activeTheme(theme: ThemePreference): Theme {
  return theme === "system" ? systemTheme : theme;
}

function activeLocale(locale: LocalePreference): AppLocale {
  return locale === "system" ? osLocale : locale;
}

async function persist(state: Preferences): Promise<void> {
  const store = await load(SETTINGS_FILE, { autoSave: false });
  await store.set(SETTINGS_KEY, { schemaVersion: PREFERENCES_SCHEMA_VERSION, value: state });
  await store.save();
}

async function syncSystemLocale(locale: LocalePreference): Promise<void> {
  await invokeCommand("plugin:desktop|set_locale", { locale: activeLocale(locale) });
}

function applyPreferences(state: Preferences): void {
  const root = document.documentElement;
  const locale = activeLocale(state.locale);
  root.dataset.theme = activeTheme(state.theme);
  root.dataset.density = state.density;
  root.dataset.textSize = state.textSize;
  root.lang = locale;
  i18n.global.locale.value = locale;
}

export const usePreferencesStore = defineStore("preferences", () => {
  const preferences = reactive(snapshot(defaultPreferences));
  const serialize = pLimit(1);
  let loaded = false;
  let dirty = false;

  function loadPreferences() {
    return serialize(async () => {
      loaded = false;
      const store = await load(SETTINGS_FILE, { autoSave: false });
      const [theme, locale, raw] = await Promise.all([
        currentWindow.theme(),
        getOsLocale(),
        store.get<unknown>(SETTINGS_KEY),
      ]);

      systemTheme = theme ?? "light";
      osLocale = systemLocale(locale);

      if (!isNil(raw)) {
        Object.assign(preferences, decodePreferences(raw));
      }

      await currentWindow.setTheme(preferences.theme === "system" ? null : preferences.theme);
      await syncSystemLocale(preferences.locale);
      await invokeCommand("plugin:desktop|set_task_notifications", {
        enabled: preferences.backgroundTaskNotifications,
      });
      applyPreferences(snapshot(preferences));

      if (!stopThemeListener) {
        stopThemeListener = await currentWindow.onThemeChanged(({ payload }) => {
          systemTheme = payload;
          if (preferences.theme === "system") applyPreferences(snapshot(preferences));
        });
      }
      loaded = true;
      dirty = false;
    });
  }

  function mutate(change: () => void | Promise<void>): Promise<void> {
    if (isApplicationTransitioning()) return Promise.reject(new Error("Preferences are transitioning"));
    return serialize(async () => {
      await change();
      dirty = true;
      const state = snapshot(preferences);
      applyPreferences(state);
      await persist(state);
      dirty = false;
    });
  }

  function flush(): Promise<void> {
    return serialize(async () => {
      if (!loaded || !dirty) return;
      await persist(snapshot(preferences));
      dirty = false;
    });
  }

  function setTheme(theme: ThemePreference) {
    return mutate(async () => {
      await currentWindow.setTheme(theme === "system" ? null : theme);
      preferences.theme = theme;
    });
  }

  function setDensity(density: DensityPreference) {
    return mutate(() => {
      preferences.density = density;
    });
  }

  function setTextSize(textSize: TextSizePreference) {
    return mutate(() => {
      preferences.textSize = textSize;
    });
  }

  function setLocale(locale: LocalePreference) {
    return mutate(async () => {
      await syncSystemLocale(locale);
      preferences.locale = locale;
    });
  }

  function setWorkbenchLayout(layout: WorkbenchPreferences) {
    return mutate(() => {
      Object.assign(preferences.workbench, layout);
    });
  }

  function setBackgroundTaskNotifications(enabled: boolean) {
    return mutate(async () => {
      await invokeCommand("plugin:desktop|set_task_notifications", { enabled });
      preferences.backgroundTaskNotifications = enabled;
    });
  }

  return {
    ...toRefs(preferences),
    load: loadPreferences,
    flush,
    setTheme,
    setDensity,
    setTextSize,
    setLocale,
    setWorkbenchLayout,
    setBackgroundTaskNotifications,
  };
});

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopThemeListener?.();
    stopThemeListener = undefined;
  });
}
