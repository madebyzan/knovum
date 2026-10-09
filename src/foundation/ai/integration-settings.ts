import { error as logError } from "@tauri-apps/plugin-log";
import { LazyStore } from "@tauri-apps/plugin-store";
import { cloneDeep, isNil } from "es-toolkit";
import pLimit from "p-limit";
import { z } from "zod";

import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import {
  creationLibrarySchema,
  defaultCreationLibrary,
  type CreationLibrary,
} from "./schema/creation-library";
import { aiModelSettingsSchema, defaultAiModelSettings, type AiModelSettings } from "./schema/model-settings";

const SETTINGS_FILE = "settings.json";
const settingKeys = { aiModels: "ai.models", creationLibrary: "creation.library" } as const;
const SETTINGS_SCHEMA_VERSION = 1;
const settingsStore = new LazyStore(SETTINGS_FILE, { autoSave: false });

const serialize = pLimit(1);

function decodeSetting<T>(raw: unknown, schema: z.ZodType<T>): T {
  return z
    .object({ schemaVersion: z.literal(SETTINGS_SCHEMA_VERSION), value: schema })
    .strict()
    .parse(raw).value;
}
export function decodeAiModelSettings(raw: unknown): AiModelSettings {
  return decodeSetting(raw, aiModelSettingsSchema);
}
async function readSetting<T>(key: string, schema: z.ZodType<T>, defaultValue: T): Promise<T> {
  return serialize(async () => {
    const raw = await settingsStore.get<unknown>(key);
    if (isNil(raw)) return cloneDeep(defaultValue);
    return decodeSetting(raw, schema);
  });
}
function persistSetting<T>(key: string, value: T): Promise<void> {
  return serialize(async () => {
    await settingsStore.set(key, { schemaVersion: SETTINGS_SCHEMA_VERSION, value });
    await settingsStore.save();
  }).catch((error) => {
    void logError(`Integration settings mutation failed: ${formatErrorDiagnostic(error)}`);
    throw error;
  });
}

export function loadAiModelSettings() {
  return readSetting(settingKeys.aiModels, aiModelSettingsSchema, defaultAiModelSettings);
}
export async function saveAiModelSettings(value: AiModelSettings) {
  const parsed = aiModelSettingsSchema.parse(value);
  await invokeCommand("plugin:ai|validate_model_settings", { settings: parsed });
  return persistSetting(settingKeys.aiModels, parsed);
}
export function resetAiModelSettings() {
  return persistSetting(settingKeys.aiModels, defaultAiModelSettings);
}
export function loadCreationLibrary() {
  return readSetting(settingKeys.creationLibrary, creationLibrarySchema, defaultCreationLibrary);
}
export async function saveCreationLibrary(value: CreationLibrary) {
  const parsed = creationLibrarySchema.parse(value);
  return persistSetting(settingKeys.creationLibrary, parsed);
}
export function resetCreationLibrary() {
  return persistSetting(settingKeys.creationLibrary, defaultCreationLibrary);
}
