import { getCurrentWindow, type Theme } from "@tauri-apps/api/window";
import { locale as getOsLocale } from "@tauri-apps/plugin-os";
import { load } from "@tauri-apps/plugin-store";
import { pick } from "es-toolkit";
import { defineStore } from "pinia";

import { i18n, systemLocale, type AppLocale } from "../i18n/i18n";
import { invokeCommand as invoke } from "../ipc/invoke-command";
import {
  decodePreferences,
  defaultPreferences,
  PREFERENCES_SCHEMA_VERSION,
  type DensityPreference,
  type LocalePreference,
  type Preferences,
  type TextSizePreference,
  type ThemePreference,
} from "./preferences-schema";

export type { DensityPreference, LocalePreference, TextSizePreference, ThemePreference };

const SETTINGS_FILE = "settings.json";
const SETTINGS_KEY = "preferences";
const currentWindow = getCurrentWindow();
let systemTheme: Theme = "light";
let osLocale: AppLocale = "en";
let stopThemeListener: (() => void) | undefined;

function snapshot(state: Preferences): Preferences {
  return pick(state, ["theme", "density", "textSize", "locale"]);
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
  await invoke("plugin:desktop|set_locale", { locale: activeLocale(locale) });
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

export const usePreferencesStore = defineStore("preferences", {
  state: () => ({ ...defaultPreferences }),
  actions: {
    async load() {
      const store = await load(SETTINGS_FILE, { autoSave: false });
      const [theme, locale, raw] = await Promise.all([
        currentWindow.theme(),
        getOsLocale(),
        store.get<unknown>(SETTINGS_KEY),
      ]);

      systemTheme = theme ?? "light";
      osLocale = systemLocale(locale);

      if (raw !== undefined && raw !== null) {
        Object.assign(this, decodePreferences(raw));
      }

      await Promise.all([
        currentWindow.setTheme(this.theme === "system" ? null : this.theme),
        syncSystemLocale(this.locale),
      ]);
      applyPreferences(snapshot(this));

      if (!stopThemeListener) {
        stopThemeListener = await currentWindow.onThemeChanged(({ payload }) => {
          systemTheme = payload;
          if (this.theme === "system") applyPreferences(snapshot(this));
        });
      }
    },
    async setTheme(theme: ThemePreference) {
      this.theme = theme;
      await currentWindow.setTheme(theme === "system" ? null : theme);
      const state = snapshot(this);
      applyPreferences(state);
      await persist(state);
    },
    async setDensity(density: DensityPreference) {
      this.density = density;
      const state = snapshot(this);
      applyPreferences(state);
      await persist(state);
    },
    async setTextSize(textSize: TextSizePreference) {
      this.textSize = textSize;
      const state = snapshot(this);
      applyPreferences(state);
      await persist(state);
    },
    async setLocale(locale: LocalePreference) {
      this.locale = locale;
      const state = snapshot(this);
      applyPreferences(state);
      await Promise.all([syncSystemLocale(locale), persist(state)]);
    },
  },
});

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopThemeListener?.();
    stopThemeListener = undefined;
  });
}
