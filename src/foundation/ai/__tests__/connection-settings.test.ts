import { beforeEach, expect, it, vi } from "vitest";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import { runApplicationTransition } from "../../lifecycle/application-transition";
import { removeAiConnection, saveAiConnection } from "../connection-settings";
import { flushAiSettingsMutations, resetAiSettings } from "../settings-mutations";

const state = vi.hoisted(() => ({
  settings: { connections: [] as AiConnectionSettings[] },
  flush: vi.fn<() => Promise<void>>(),
  reset: vi.fn<() => Promise<void>>(),
}));
const secrets = vi.hoisted(() => ({
  deleteSecret: vi.fn<() => Promise<void>>(),
  persistOptionalSecret: vi.fn<() => Promise<void>>(),
}));
vi.mock("../ai-settings-store", () => ({ useAiSettingsStore: () => state }));
vi.mock("../../secrets/secret-store", () => secrets);
const connection: AiConnectionSettings = {
  id: "connection",
  name: "Provider",
  provider: "openai",
  baseUrl: "https://api.openai.com",
  models: [],
};

beforeEach(() => {
  vi.resetAllMocks();
  state.settings.connections = [structuredClone(connection)];
  state.flush.mockResolvedValue(undefined);
  state.reset.mockResolvedValue(undefined);
  secrets.deleteSecret.mockResolvedValue(undefined);
  secrets.persistOptionalSecret.mockResolvedValue(undefined);
});

it("does not change credentials when configuration persistence fails", async () => {
  const failure = new Error("settings failed");
  state.flush.mockRejectedValueOnce(failure);
  await expect(saveAiConnection({ ...connection, name: "Changed" }, "new-key")).rejects.toBe(failure);
  expect(state.settings.connections[0]?.name).toBe("Provider");
  expect(secrets.persistOptionalSecret).not.toHaveBeenCalled();
});

it("retains committed settings and reports a failed credential write", async () => {
  const failure = new Error("secret failed");
  secrets.persistOptionalSecret.mockRejectedValueOnce(failure);
  await expect(saveAiConnection({ ...connection, name: "Changed" }, "new-key")).resolves.toEqual({
    status: "credential-failed",
    cause: failure,
  });
  expect(state.settings.connections[0]?.name).toBe("Changed");
});

it("preserves the original credential failure and rollback failure", async () => {
  const original = new Error("delete failed");
  const rollback = new Error("restore failed");
  secrets.deleteSecret.mockRejectedValueOnce(original);
  state.flush.mockResolvedValueOnce(undefined).mockRejectedValueOnce(rollback);
  const operation = removeAiConnection(connection.id);
  await expect(operation).rejects.toBeInstanceOf(AggregateError);
  await expect(operation).rejects.toMatchObject({ errors: [original, rollback], cause: original });
});

it("does not delete credentials or retry persistence when the initial removal fails", async () => {
  const failure = new Error("settings failed");
  state.flush.mockRejectedValueOnce(failure);
  await expect(removeAiConnection(connection.id)).rejects.toBe(failure);
  expect(state.settings.connections).toHaveLength(1);
  expect(state.flush).toHaveBeenCalledOnce();
  expect(secrets.deleteSecret).not.toHaveBeenCalled();
});

it("serializes another edit until credential work completes", async () => {
  let release!: () => void;
  secrets.persistOptionalSecret.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      release = resolve;
    }),
  );
  const first = saveAiConnection({ ...connection, name: "First" }, "key");
  await vi.waitFor(() => expect(secrets.persistOptionalSecret).toHaveBeenCalledOnce());
  const second = saveAiConnection({ ...connection, name: "Second" }, undefined);
  await Promise.resolve();
  expect(state.settings.connections[0]?.name).toBe("First");
  release();
  await first;
  await second;
  expect(state.settings.connections[0]?.name).toBe("Second");
});

it("waits for credential completion before reset or exit flushing", async () => {
  let release!: () => void;
  secrets.persistOptionalSecret.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      release = resolve;
    }),
  );
  const saving = saveAiConnection(connection, "key");
  await vi.waitFor(() => expect(secrets.persistOptionalSecret).toHaveBeenCalledOnce());
  const resetting = resetAiSettings();
  const closing = flushAiSettingsMutations();
  expect(state.reset).not.toHaveBeenCalled();
  expect(state.flush).toHaveBeenCalledOnce();
  release();
  await Promise.all([saving, resetting, closing]);
  expect(state.reset).toHaveBeenCalledOnce();
  expect(state.flush).toHaveBeenCalledTimes(2);
});

it("rejects new edits during an application transition while allowing its save barrier", async () => {
  await runApplicationTransition(async () => {
    await expect(saveAiConnection(connection, "key")).rejects.toThrow("transitioning");
    await flushAiSettingsMutations();
  });
  expect(secrets.persistOptionalSecret).not.toHaveBeenCalled();
  expect(state.flush).toHaveBeenCalledOnce();
});
