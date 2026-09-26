import { error as logError } from "@tauri-apps/plugin-log";
import { LazyStore } from "@tauri-apps/plugin-store";
import { z } from "zod";

import { invokeCommand as invoke } from "../ipc/invoke-command";
import { aiModelSettingsSchema, defaultAiModelSettings, type AiModelSettings } from "./ai-model-settings";
import { creationLibrarySchema, defaultCreationLibrary, type CreationLibrary } from "./creation-library";

export { defaultAiModelSettings, type AiModelSettings } from "./ai-model-settings";
export { defaultCreationLibrary, type CreationLibrary } from "./creation-library";

export type { AiProvider } from "./ai-models";

const SETTINGS_FILE = "settings.json";
const settingKeys = { aiModels: "ai.models", creationLibrary: "creation.library" } as const;
const SETTINGS_SCHEMA_VERSION = 1;
const settingsStore = new LazyStore(SETTINGS_FILE, { autoSave: false });

let writeQueue: Promise<void> = Promise.resolve();

function enqueueMutation(mutation: () => Promise<void>): Promise<void> {
  const operation = writeQueue.then(mutation);
  writeQueue = operation.catch((error) => {
    void logError(`Integration settings mutation failed: ${String(error)}`);
  });
  return operation;
}

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
  await writeQueue;
  const raw = await settingsStore.get<unknown>(key);
  if (raw === undefined || raw === null) return structuredClone(defaultValue);
  return decodeSetting(raw, schema);
}
function writeSetting<T>(key: string, schema: z.ZodType<T>, value: T): Promise<void> {
  const parsed = schema.parse(value);
  return enqueueMutation(async () => {
    await settingsStore.set(key, { schemaVersion: SETTINGS_SCHEMA_VERSION, value: parsed });
    await settingsStore.save();
  });
}

export const loadAiModelSettings = () =>
  readSetting(settingKeys.aiModels, aiModelSettingsSchema, defaultAiModelSettings);
export async function saveAiModelSettings(value: AiModelSettings) {
  const parsed = aiModelSettingsSchema.parse(value);
  await invoke("plugin:ai|validate_model_settings", { settings: parsed });
  return writeSetting(settingKeys.aiModels, aiModelSettingsSchema, parsed);
}
export const resetAiModelSettings = () =>
  writeSetting(settingKeys.aiModels, aiModelSettingsSchema, defaultAiModelSettings);
export const loadCreationLibrary = () =>
  readSetting(settingKeys.creationLibrary, creationLibrarySchema, defaultCreationLibrary);
export async function saveCreationLibrary(value: CreationLibrary) {
  const parsed = creationLibrarySchema.parse(value);
  return writeSetting(settingKeys.creationLibrary, creationLibrarySchema, parsed);
}
export const resetCreationLibrary = () =>
  writeSetting(settingKeys.creationLibrary, creationLibrarySchema, defaultCreationLibrary);
