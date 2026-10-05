import { beforeEach, expect, it, vi } from "vitest";

import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../shared/contracts/generated/AiModelDefinition";
import { removeAiConnectionModel, saveAiConnectionModel } from "../connection-settings";

const state = vi.hoisted(() => ({
  settings: { connections: [] as AiConnectionSettings[] },
  flush: vi.fn<() => Promise<void>>(),
}));
vi.mock("../ai-settings-store", () => ({ useAiSettingsStore: () => state }));
vi.mock("../../secrets/secret-store", () => ({
  deleteSecret: vi.fn<() => Promise<void>>(),
  persistOptionalSecret: vi.fn<() => Promise<void>>(),
}));
const model: AiModelDefinition = {
  id: "model",
  model: "remote",
  name: "Model",
  capabilities: ["text-generation"],
};
beforeEach(() => {
  vi.resetAllMocks();
  state.settings.connections = [
    {
      id: "connection",
      name: "Provider",
      provider: "openai",
      baseUrl: "https://api.openai.com",
      models: [structuredClone(model)],
    },
  ];
  state.flush.mockResolvedValue(undefined);
});

it("persists a model edit and restores the original when saving fails", async () => {
  await saveAiConnectionModel("connection", { ...model, name: "Edited" });
  const failure = new Error("persistence failed");
  state.flush.mockRejectedValueOnce(failure);
  await expect(saveAiConnectionModel("connection", { ...model, name: "Failed" })).rejects.toBe(failure);
  expect(state.settings.connections[0]?.models[0]?.name).toBe("Edited");
});

it("restores a deleted model on failure and allows a later successful removal", async () => {
  state.flush.mockRejectedValueOnce(new Error("persistence failed"));
  await expect(removeAiConnectionModel("connection", model.id)).rejects.toThrow("persistence failed");
  expect(state.settings.connections[0]?.models).toHaveLength(1);
  await removeAiConnectionModel("connection", model.id);
  expect(state.settings.connections[0]?.models).toHaveLength(0);
});
