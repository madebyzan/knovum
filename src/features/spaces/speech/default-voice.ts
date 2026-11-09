import { error as logError } from "@tauri-apps/plugin-log";
import { LazyStore } from "@tauri-apps/plugin-store";
import { isNil } from "es-toolkit";
import pLimit from "p-limit";
import { shallowRef } from "vue";
import { z } from "zod";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";

const SETTINGS_FILE = "settings.json";
const SETTING_KEY = "speech.defaultVoiceProfileId";
const SETTINGS_SCHEMA_VERSION = 1;
const settingsStore = new LazyStore(SETTINGS_FILE, { autoSave: false });

const serialize = pLimit(1);
const defaultVoiceProfileId = shallowRef<string | null>(null);
let initialized = false;

export async function loadDefaultVoiceProfileId(): Promise<string | null> {
  return serialize(async () => {
    try {
      const raw = await settingsStore.get<unknown>(SETTING_KEY);
      if (isNil(raw)) {
        defaultVoiceProfileId.value = null;
        initialized = true;
        return null;
      }
      const parsed = z
        .object({ schemaVersion: z.literal(SETTINGS_SCHEMA_VERSION), value: z.string().nullable() })
        .strict()
        .parse(raw).value;
      defaultVoiceProfileId.value = parsed;
      initialized = true;
      return parsed;
    } catch {
      defaultVoiceProfileId.value = null;
      initialized = true;
      return null;
    }
  });
}

export async function saveDefaultVoiceProfileId(id: string | null): Promise<void> {
  return serialize(async () => {
    try {
      await settingsStore.set(SETTING_KEY, { schemaVersion: SETTINGS_SCHEMA_VERSION, value: id });
      await settingsStore.save();
      defaultVoiceProfileId.value = id;
    } catch (error) {
      void logError(`Speech default voice preference mutation failed: ${formatErrorDiagnostic(error)}`);
      throw error;
    }
  });
}

export function useDefaultVoiceProfile() {
  if (!initialized) {
    void loadDefaultVoiceProfileId();
  }
  return {
    defaultVoiceProfileId,
    setDefaultVoiceProfile: saveDefaultVoiceProfileId,
  };
}
