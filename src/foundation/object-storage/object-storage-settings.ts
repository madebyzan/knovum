import { LazyStore } from "@tauri-apps/plugin-store";
import { isNil } from "es-toolkit";
import { z } from "zod";

import type { ObjectStorageSettings } from "../../shared/contracts/generated/ObjectStorageSettings";
import { invokeCommand } from "../ipc/invoke-command";
import { deleteSecret } from "../secrets/secret-store";
import { objectStorageSettingsSchema } from "./schema/object-storage-settings";

const SETTINGS_FILE = "settings.json";
const SETTINGS_KEY = "storage.objectStorage";
const SETTINGS_SCHEMA_VERSION = 1;

export const OBJECT_STORAGE_ACCESS_KEY_ID = "integrations.storage.object-storage.access-key-id";
export const OBJECT_STORAGE_SECRET_ACCESS_KEY = "integrations.storage.object-storage.secret-access-key";

const settingsStore = new LazyStore(SETTINGS_FILE, { autoSave: false });

export async function loadObjectStorageSettings(): Promise<ObjectStorageSettings | null> {
  const raw = await settingsStore.get<unknown>(SETTINGS_KEY);
  if (isNil(raw)) return null;
  return z
    .object({ schemaVersion: z.literal(SETTINGS_SCHEMA_VERSION), value: objectStorageSettingsSchema })
    .strict()
    .parse(raw).value;
}

export async function saveObjectStorageSettings(settings: ObjectStorageSettings): Promise<void> {
  const parsed = objectStorageSettingsSchema.parse(settings);
  await invokeCommand("plugin:storage|validate_object_storage_settings", { settings: parsed });
  await settingsStore.set(SETTINGS_KEY, { schemaVersion: SETTINGS_SCHEMA_VERSION, value: parsed });
  await settingsStore.save();
}

export async function removeObjectStorageConfiguration(): Promise<void> {
  await settingsStore.delete(SETTINGS_KEY);
  await settingsStore.save();
  await deleteSecret(OBJECT_STORAGE_ACCESS_KEY_ID);
  await deleteSecret(OBJECT_STORAGE_SECRET_ACCESS_KEY);
}

export function testObjectStorageConnection(): Promise<void> {
  return invokeCommand("plugin:storage|test_object_storage_connection");
}
