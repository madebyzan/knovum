import { getCurrentWindow, type Theme } from "@tauri-apps/api/window";
import { locale as getOsLocale } from "@tauri-apps/plugin-os";
import { load } from "@tauri-apps/plugin-store";
import { isNil, pick } from "es-toolkit";
import pLimit from "p-limit";
import { defineStore } from "pinia";
import { reactive, toRefs } from "vue";

import {
  preferenceDefaults,
  preferencesSchemaVersion,
} from "../../shared/contracts/generated/contract-values";
import { ensureLocaleLoaded, i18n, systemLocale, type AppLocale } from "../i18n/instance";
import { invokeCommand } from "../ipc/invoke-command";
import { isApplicationTransitioning } from "../lifecycle/application-transition";
import {
  decodePreferences,
  type CanvasBackgroundPattern,
  type DensityPreference,
  type LocalePreference,
  type Preferences,
  type TextSizePreference,
  type ThemePreference,
  type WorkbenchPreferences,
} from "./schema/preferences";

const SETTINGS_FILE = "settings.json";
const SETTINGS_KEY = "preferences";
let cachedWindow: ReturnType<typeof getCurrentWindow> | undefined;
let systemTheme: Theme = "light";
let osLocale: AppLocale = "en";
let stopThemeListener: (() => void) | undefined;

function currentWindow() {
  return (cachedWindow ??= getCurrentWindow());
}

function snapshot(state: Preferences): Preferences {
  return {
    ...pick(state, [
      "theme",
      "density",
      "textSize",
      "locale",
      "backgroundTaskNotifications",
      "restorePreviousWorkspace",
    ]),
    workbench: { ...state.workbench },
    canvas: { ...state.canvas },
  };
}

function activeTheme(theme: ThemePreference): "light" | "dark" | "paper" {
  return theme === "system" ? systemTheme : theme;
}

function nativeWindowTheme(theme: ThemePreference): Theme | null {
  if (theme === "system") return null;
  return theme === "paper" ? "light" : theme;
}

function activeLocale(locale: LocalePreference): AppLocale {
  return locale === "system" ? osLocale : locale;
}

async function persist(state: Preferences): Promise<void> {
  const store = await load(SETTINGS_FILE, { autoSave: false });
  await store.set(SETTINGS_KEY, { schemaVersion: preferencesSchemaVersion, value: state });
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
  const preferences = reactive(snapshot(preferenceDefaults));
  const serialize = pLimit(1);
  let loaded = false;
  let dirty = false;

  function loadPreferences() {
    return serialize(async () => {
      loaded = false;
      const store = await load(SETTINGS_FILE, { autoSave: false });
      const window = currentWindow();
      const [theme, locale, raw] = await Promise.all([
        window.theme(),
        getOsLocale(),
        store.get<unknown>(SETTINGS_KEY),
      ]);

      systemTheme = theme ?? "light";
      osLocale = systemLocale(locale);

      if (!isNil(raw)) {
        Object.assign(preferences, decodePreferences(raw));
      }

      await Promise.all([
        window.setTheme(nativeWindowTheme(preferences.theme)),
        syncSystemLocale(preferences.locale),
        syncTaskNotifications(preferences.backgroundTaskNotifications),
        ensureLocaleLoaded(activeLocale(preferences.locale)),
      ]);
      applyPreferences(snapshot(preferences));

      if (!stopThemeListener) {
        stopThemeListener = await window.onThemeChanged(({ payload }) => {
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
      await currentWindow().setTheme(nativeWindowTheme(theme));
      preferences.theme = theme;
    });
  }

  const setDensity = (density: DensityPreference) => mutate(() => void (preferences.density = density));
  const setTextSize = (textSize: TextSizePreference) => mutate(() => void (preferences.textSize = textSize));
  const setWorkbenchLayout = (layout: WorkbenchPreferences) =>
    mutate(() => void Object.assign(preferences.workbench, layout));
  const setCanvasBackgroundPattern = (pattern: CanvasBackgroundPattern) =>
    mutate(() => void (preferences.canvas.backgroundPattern = pattern));
  const setRestorePreviousWorkspace = (enabled: boolean) =>
    mutate(() => void (preferences.restorePreviousWorkspace = enabled));

  function setLocale(locale: LocalePreference) {
    return mutate(async () => {
      await Promise.all([syncSystemLocale(locale), ensureLocaleLoaded(activeLocale(locale))]);
      preferences.locale = locale;
    });
  }

  function setBackgroundTaskNotifications(enabled: boolean) {
    return mutate(async () => {
      await syncTaskNotifications(enabled);
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
    setCanvasBackgroundPattern,
    setBackgroundTaskNotifications,
    setRestorePreviousWorkspace,
  };
});

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopThemeListener?.();
    stopThemeListener = undefined;
  });
}

/** Desktop task notifications follow the preference through their explicit enable and disable intents. */
function syncTaskNotifications(enabled: boolean) {
  return invokeCommand(
    enabled ? "plugin:desktop|enable_task_notifications" : "plugin:desktop|disable_task_notifications",
  );
}
