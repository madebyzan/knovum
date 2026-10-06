import type { AiConnectionSettings } from "../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../shared/contracts/generated/AiModelDefinition";
import { deleteSecret, persistOptionalSecret } from "../secrets/secret-store";
import { useAiSettingsStore } from "./ai-settings-store";
import { aiConnectionSecretKey } from "./schema/models";
import { mutateAiSettings } from "./settings-mutations";

export type AiConnectionSaveOutcome = { status: "saved" } | { status: "credential-failed"; cause: unknown };

async function saveAiConnectionChange(
  connection: AiConnectionSettings,
  secret: string | null | undefined,
): Promise<AiConnectionSaveOutcome> {
  const store = useAiSettingsStore();
  const connections = store.settings.connections;
  const index = connections.findIndex(({ id }) => id === connection.id);
  const previous = connections[index];
  if (index >= 0) connections[index] = connection;
  else connections.push(connection);
  try {
    await store.flush();
  } catch (cause) {
    if (previous) connections[index] = previous;
    else connections.splice(connections.indexOf(connection), 1);
    throw cause;
  }
  try {
    const key = aiConnectionSecretKey(connection.id);
    if (secret === null) await deleteSecret(key);
    else if (secret !== undefined) await persistOptionalSecret(key, secret);
    return { status: "saved" };
  } catch (cause) {
    return { status: "credential-failed", cause };
  }
}

async function removeAiConnectionChange(connectionId: string): Promise<void> {
  const store = useAiSettingsStore();
  const connections = store.settings.connections;
  const index = connections.findIndex((connection) => connection.id === connectionId);
  const connection = connections[index];
  if (!connection) throw new Error("AI connection does not exist");
  connections.splice(index, 1);
  try {
    await store.flush();
  } catch (cause) {
    connections.splice(index, 0, connection);
    throw cause;
  }
  try {
    await deleteSecret(aiConnectionSecretKey(connectionId));
  } catch (cause) {
    connections.splice(index, 0, connection);
    try {
      await store.flush();
    } catch (rollbackError) {
      throw new AggregateError(
        [cause, rollbackError],
        "Deleting the connection credential and restoring its configuration both failed",
        { cause },
      );
    }
    throw cause;
  }
}

function connectionModels(connectionId: string) {
  const connection = useAiSettingsStore().settings.connections.find(({ id }) => id === connectionId);
  if (!connection) throw new Error("AI connection does not exist");
  return connection.models;
}

async function saveAiConnectionModelChange(connectionId: string, model: AiModelDefinition): Promise<void> {
  const models = connectionModels(connectionId);
  const index = models.findIndex(({ id }) => id === model.id);
  const previous = models[index];
  if (index >= 0) models[index] = model;
  else models.push(model);
  try {
    await useAiSettingsStore().flush();
  } catch (cause) {
    if (previous) models[index] = previous;
    else models.splice(models.indexOf(model), 1);
    throw cause;
  }
}

async function removeAiConnectionModelChange(connectionId: string, modelId: string): Promise<void> {
  const models = connectionModels(connectionId);
  const index = models.findIndex(({ id }) => id === modelId);
  const model = models[index];
  if (!model) throw new Error("AI connection model does not exist");
  models.splice(index, 1);
  try {
    await useAiSettingsStore().flush();
  } catch (cause) {
    models.splice(index, 0, model);
    throw cause;
  }
}

export function saveAiConnection(connection: AiConnectionSettings, secret: string | null | undefined) {
  return mutateAiSettings(() => saveAiConnectionChange(connection, secret));
}
export function removeAiConnection(connectionId: string) {
  return mutateAiSettings(() => removeAiConnectionChange(connectionId));
}
export function saveAiConnectionModel(connectionId: string, model: AiModelDefinition) {
  return mutateAiSettings(() => saveAiConnectionModelChange(connectionId, model));
}
export function removeAiConnectionModel(connectionId: string, modelId: string) {
  return mutateAiSettings(() => removeAiConnectionModelChange(connectionId, modelId));
}
