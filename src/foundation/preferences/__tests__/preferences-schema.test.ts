import { describe, expect, it } from "vitest";

import { decodePreferences } from "../schema/preferences";

const current = {
  theme: "system",
  density: "compact",
  textSize: "default",
  locale: "system",
} as const;

describe("preferences schema", () => {
  it("accepts only the current persisted envelope", () => {
    expect(decodePreferences({ schemaVersion: 1, value: current })).toEqual(current);
  });

  it("rejects any persisted settings that are not the version-1 envelope", () => {
    expect(() => decodePreferences(current)).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
  });

  it("rejects unknown persisted schemas", () => {
    expect(() => decodePreferences({ schemaVersion: 2, value: current })).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
  });
  it("rejects extra fields instead of accepting a compatible superset", () => {
    expect(() => decodePreferences({ schemaVersion: 1, value: current, previousVersion: 1 })).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
    expect(() => decodePreferences({ schemaVersion: 1, value: { ...current, legacyTheme: "dark" } })).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
  });
});
