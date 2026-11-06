import { error as logError } from "@tauri-apps/plugin-log";
import { LazyStore } from "@tauri-apps/plugin-store";
import { isNil } from "es-toolkit";
import pLimit from "p-limit";
import { defineStore } from "pinia";
import { reactive, shallowRef } from "vue";
import { z } from "zod";

import type { DictationShortcutStatus } from "../../shared/contracts/generated/DictationShortcutStatus";
import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { shortcutSettingsSchema, type ShortcutSettings } from "./schema/shortcut-settings";
import { toGlobalAccelerator } from "./shortcut-chord";

const SETTINGS_FILE = "settings.json";
const SETTING_KEY = "shortcuts";
const SETTINGS_SCHEMA_VERSION = 1;
const settingsStore = new LazyStore(SETTINGS_FILE, { autoSave: false });
const serialize = pLimit(1);

function decodeShortcutSettings(raw: unknown): ShortcutSettings {
  return z
    .object({ schemaVersion: z.literal(SETTINGS_SCHEMA_VERSION), value: shortcutSettingsSchema })
    .strict()
    .parse(raw).value;
}

export const useShortcutsStore = defineStore("shortcuts", () => {
  const customShortcuts = reactive<Record<string, string>>({});
  const dictationStatus = shallowRef<DictationShortcutStatus>();
  let loaded = false;

  async function load() {
    return serialize(async () => {
      try {
        const raw = await settingsStore.get<unknown>(SETTING_KEY);
        if (!isNil(raw)) {
          const decoded = decodeShortcutSettings(raw);
          for (const key of Object.keys(customShortcuts)) delete customShortcuts[key];
          Object.assign(customShortcuts, decoded.custom);
        }
      } catch (error) {
        void logError(`Failed to load shortcut preferences: ${formatErrorDiagnostic(error)}`);
      }
      try {
        if (customShortcuts["dictation"]) {
          const accelerator = toGlobalAccelerator(customShortcuts["dictation"]);
          dictationStatus.value = await invokeCommand<DictationShortcutStatus>(
            "plugin:dictation|set_dictation_shortcut",
            { shortcut: accelerator },
          );
        } else {
          dictationStatus.value = await invokeCommand<DictationShortcutStatus>(
            "plugin:dictation|get_dictation_shortcut",
          );
        }
      } catch (error) {
        void logError(`Failed to sync dictation shortcut: ${formatErrorDiagnostic(error)}`);
      }
      loaded = true;
    });
  }

  async function persist(): Promise<void> {
    const value: ShortcutSettings = { custom: { ...customShortcuts } };
    await settingsStore.set(SETTING_KEY, { schemaVersion: SETTINGS_SCHEMA_VERSION, value });
    await settingsStore.save();
  }

  function getEffectiveShortcut(commandId: string, defaultShortcut?: string): string | undefined {
    return customShortcuts[commandId] ?? defaultShortcut;
  }

  function isCustomized(commandId: string): boolean {
    return commandId in customShortcuts;
  }

  async function setCustomShortcut(commandId: string, chord: string) {
    customShortcuts[commandId] = chord;
    await persist();
  }

  async function resetCustomShortcut(commandId: string) {
    if (commandId in customShortcuts) {
      delete customShortcuts[commandId];
      await persist();
    }
  }

  async function setDictationShortcut(chord: string) {
    const accelerator = toGlobalAccelerator(chord);
    const status = await invokeCommand<DictationShortcutStatus>("plugin:dictation|set_dictation_shortcut", {
      shortcut: accelerator,
    });
    dictationStatus.value = status;
    customShortcuts["dictation"] = chord;
    await persist();
  }

  async function resetDictationShortcut() {
    const status = await invokeCommand<DictationShortcutStatus>("plugin:dictation|reset_dictation_shortcut");
    dictationStatus.value = status;
    delete customShortcuts["dictation"];
    await persist();
  }

  async function resetAll() {
    for (const key of Object.keys(customShortcuts)) delete customShortcuts[key];
    await persist();
    try {
      dictationStatus.value = await invokeCommand<DictationShortcutStatus>(
        "plugin:dictation|reset_dictation_shortcut",
      );
    } catch {
      // Best-effort reset on unsupported platforms
    }
  }

  return {
    customShortcuts,
    dictationStatus,
    loaded: () => loaded,
    load,
    getEffectiveShortcut,
    isCustomized,
    setCustomShortcut,
    resetCustomShortcut,
    setDictationShortcut,
    resetDictationShortcut,
    resetAll,
  };
});
