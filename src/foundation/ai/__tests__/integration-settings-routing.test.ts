import { describe, expect, it } from "vitest";

import { defaultExecutionTarget } from "../ai-execution";
import { decodeAiModelSettings } from "../integration-settings";
import { DEFAULT_AI_MODEL_SETTINGS } from "../schema/model-settings";

function configuredSettings() {
  const value = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
  value.connections = [
    {
      id: "user-model",
      name: "My model",
      provider: "openai",
      baseUrl: "https://api.openai.com/v1",
      models: [{ id: "text", name: "Text", model: "gpt-4.1", capabilities: ["text-generation"] }],
    },
  ];
  value.desktopAgents = [{ kind: "codex", enabled: true }];
  value.defaults.writing = { type: "model", connectionId: "user-model", modelId: "text" };
  value.defaults.visualUnderstanding = { type: "agent", runtimeKind: "codex" };
  return value;
}

describe("optional AI execution preferences", () => {
  it("preserves configured accounts and overrides when the global preference is absent", () => {
    const value = configuredSettings();
    const defaults: Record<string, unknown> = { ...value.defaults };
    delete defaults.defaultTarget;
    const raw = { schemaVersion: 1, value: { ...value, defaults } };
    const stored = structuredClone(raw);

    const decoded = decodeAiModelSettings(raw);

    expect(decoded).toEqual(value);
    expect(raw).toEqual(stored);
    expect(defaultExecutionTarget(decoded, "writing")).toEqual(value.defaults.writing);
    expect(defaultExecutionTarget(decoded, "visual-understanding")).toEqual(
      value.defaults.visualUnderstanding,
    );
    expect(defaultExecutionTarget(decoded, "knowledge")).toBeNull();
    expect(decodeAiModelSettings({ schemaVersion: 1, value: decoded })).toEqual(decoded);
  });

  it("does not select an installed account when all execution preferences are absent", () => {
    const value = configuredSettings();
    const decoded = decodeAiModelSettings({ schemaVersion: 1, value: { ...value, defaults: {} } });
    expect(decoded).toEqual({ ...value, defaults: DEFAULT_AI_MODEL_SETTINGS.defaults });
    expect(defaultExecutionTarget(decoded, "knowledge")).toBeNull();
  });

  it("inherits the selected global executor when a function override is absent", () => {
    const value = configuredSettings();
    const target = { type: "agent", runtimeKind: "codex" };
    const decoded = decodeAiModelSettings({
      schemaVersion: 1,
      value: { ...value, defaults: { defaultTarget: target } },
    });
    expect(defaultExecutionTarget(decoded, "knowledge")).toEqual(target);
  });

  it.each(Object.keys(DEFAULT_AI_MODEL_SETTINGS.defaults))("rejects an invalid explicit %s", (key) => {
    const value = configuredSettings();
    expect(() =>
      decodeAiModelSettings({
        schemaVersion: 1,
        value: { ...value, defaults: { [key]: { type: "agent", runtimeKind: "unknown" } } },
      }),
    ).toThrow(/Invalid option/u);
  });

  it("still rejects unknown fields, missing required data and dangling selections", () => {
    const value = configuredSettings();
    for (const invalid of [
      { ...value, defaults: { extra: null } },
      { ...value, connections: undefined },
      { ...value, defaults: undefined },
      { ...value, defaults: { defaultTarget: { type: "agent", runtimeKind: "qoder_cli" } } },
    ])
      expect(() => decodeAiModelSettings({ schemaVersion: 1, value: invalid })).toThrow(
        /Unrecognized key|Invalid input|not enabled/u,
      );
  });
});
