import { describe, expect, it } from "vitest";

import { decodePreferences } from "../schema/preferences";

const current = {
  theme: "system",
  density: "compact",
  textSize: "default",
  locale: "system",
  backgroundTaskNotifications: true,
  workbench: {
    sidebarVisible: true,
    sidebarWidth: 224,
    rightSidebarVisible: false,
    rightSidebarWidth: 380,
  },
} as const;

describe("preferences schema", () => {
  it.each(["zh-TW", "ja", "ko", "fr", "de", "es", "pt-BR", "it", "ru"])(
    "persists the selected display language %s",
    (locale) => {
      const value = { ...current, locale };
      expect(decodePreferences({ schemaVersion: 1, value })).toEqual(value);
    },
  );

  it.each(["ar", "pt", "zh-cn", "fr-FR"])("rejects an unsupported display identifier %s", (locale) => {
    expect(() => decodePreferences({ schemaVersion: 1, value: { ...current, locale } })).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
  });

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
  it("initializes absent optional preferences without changing saved choices", () => {
    const { workbench: _workbench, backgroundTaskNotifications: _notifications, ...saved } = current;
    const raw = { schemaVersion: 1, value: { ...saved, locale: "zh-CN" } };
    expect(decodePreferences(raw)).toEqual({ ...current, locale: "zh-CN" });
    expect(raw.value).not.toHaveProperty("workbench");
    expect(raw.value).not.toHaveProperty("backgroundTaskNotifications");
  });

  it("initializes each absent optional section independently", () => {
    const { workbench: _workbench, ...missingWorkbench } = current;
    const { backgroundTaskNotifications: _notifications, ...missingNotifications } = current;
    expect(decodePreferences({ schemaVersion: 1, value: missingWorkbench })).toEqual(current);
    expect(decodePreferences({ schemaVersion: 1, value: missingNotifications })).toEqual(current);
  });

  it("keeps explicit false values and fills absent layout fields", () => {
    expect(
      decodePreferences({
        schemaVersion: 1,
        value: { ...current, backgroundTaskNotifications: false, workbench: { sidebarVisible: false } },
      }),
    ).toEqual({
      ...current,
      backgroundTaskNotifications: false,
      workbench: { ...current.workbench, sidebarVisible: false },
    });
  });

  it("still rejects missing required preferences", () => {
    const { theme: _theme, ...missingTheme } = current;
    expect(() => decodePreferences({ schemaVersion: 1, value: missingTheme })).toThrow(
      "Persisted preferences use an unsupported or invalid schema.",
    );
  });

  it.each([
    { backgroundTaskNotifications: null },
    { backgroundTaskNotifications: "true" },
    { workbench: null },
    { workbench: { sidebarWidth: 199 } },
    { workbench: { rightSidebarWidth: 481 } },
    { workbench: { sidebarWidth: 224.5 } },
    { workbench: { sidebarVisible: "false" } },
    { workbench: { unknown: true } },
  ])("rejects malformed optional preferences: %j", (invalid) => {
    expect(() => decodePreferences({ schemaVersion: 1, value: { ...current, ...invalid } })).toThrow(
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
