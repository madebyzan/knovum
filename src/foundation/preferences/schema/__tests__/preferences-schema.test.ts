import { describe, expect, it } from "vitest";

import { decodePreferences } from "../preferences";

const current = {
  theme: "system",
  density: "compact",
  textSize: "default",
  locale: "system",
  backgroundTaskNotifications: true,
  restorePreviousWorkspace: true,
  workbench: {
    sidebarVisible: true,
    sidebarWidth: 224,
    rightSidebarVisible: false,
    rightSidebarWidth: 380,
  },
  canvas: {
    backgroundPattern: "dots",
    backdrop: "none",
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
    const {
      workbench: _workbench,
      canvas: _canvas,
      backgroundTaskNotifications: _notifications,
      restorePreviousWorkspace: _restore,
      ...saved
    } = current;
    const raw = { schemaVersion: 1, value: { ...saved, locale: "zh-CN" } };
    expect(decodePreferences(raw)).toEqual({ ...current, locale: "zh-CN" });
    expect(raw.value).not.toHaveProperty("workbench");
    expect(raw.value).not.toHaveProperty("canvas");
    expect(raw.value).not.toHaveProperty("backgroundTaskNotifications");
    expect(raw.value).not.toHaveProperty("restorePreviousWorkspace");
  });

  it("initializes each absent optional section independently", () => {
    const { workbench: _workbench, ...missingWorkbench } = current;
    const { canvas: _canvas, ...missingCanvas } = current;
    const { backgroundTaskNotifications: _notifications, ...missingNotifications } = current;
    const { restorePreviousWorkspace: _restore, ...missingRestore } = current;
    expect(decodePreferences({ schemaVersion: 1, value: missingWorkbench })).toEqual(current);
    expect(decodePreferences({ schemaVersion: 1, value: missingCanvas })).toEqual(current);
    expect(decodePreferences({ schemaVersion: 1, value: missingNotifications })).toEqual(current);
    expect(decodePreferences({ schemaVersion: 1, value: missingRestore })).toEqual(current);
  });

  it("keeps explicit false values and fills absent layout and canvas fields", () => {
    expect(
      decodePreferences({
        schemaVersion: 1,
        value: {
          ...current,
          backgroundTaskNotifications: false,
          restorePreviousWorkspace: false,
          workbench: { sidebarVisible: false },
          canvas: { backgroundPattern: "none" },
        },
      }),
    ).toEqual({
      ...current,
      backgroundTaskNotifications: false,
      restorePreviousWorkspace: false,
      workbench: { ...current.workbench, sidebarVisible: false },
      canvas: { ...current.canvas, backgroundPattern: "none" },
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
    { restorePreviousWorkspace: null },
    { restorePreviousWorkspace: "true" },
    { workbench: null },
    { workbench: { sidebarWidth: 199 } },
    { workbench: { rightSidebarWidth: 481 } },
    { workbench: { sidebarWidth: 224.5 } },
    { workbench: { sidebarVisible: "false" } },
    { workbench: { unknown: true } },
    { canvas: null },
    { canvas: { backgroundPattern: "stripes" } },
    { canvas: { backdrop: "neon" } },
    { canvas: { unknown: true } },
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
