import { describe, expect, it } from "vitest";

import { systemLocale } from "../instance";

describe("system display language", () => {
  it.each([
    ["zh-TW", "zh-TW"],
    ["zh-HK", "zh-TW"],
    ["zh-MO", "zh-TW"],
    ["zh-Hant-CN", "zh-TW"],
    ["zh-Hans-TW", "zh-CN"],
    ["zh-CN", "zh-CN"],
    ["zh", "zh-CN"],
    ["ja-JP", "ja"],
    ["ko-KR", "ko"],
    ["fr-CA", "fr"],
    ["de-AT", "de"],
    ["es-MX", "es"],
    ["pt-BR", "pt-BR"],
    ["pt-PT", "pt-BR"],
    ["it-IT", "it"],
    ["ru-RU", "ru"],
    ["en-GB", "en"],
    ["ar-SA", "en"],
    ["nl-NL", "en"],
    [null, "en"],
  ])("selects %s as %s", (system, display) => {
    expect(systemLocale(system)).toBe(display);
  });
});
