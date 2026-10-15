import { describe, expect, it } from "vitest";
import { createI18n } from "vue-i18n";

import { i18n as appI18n } from "../instance";
import { localeMessages } from "../locales/locale-registry";

const examples = [
  ["settings.ai.models.count", "model", "models"],
  ["spaces.inspector.itemsSelected", "item selected", "items selected"],
  ["spaces.space.nodeCount", "item", "items"],
  ["spaces.richtext.statistics.words", "word", "words"],
  ["spaces.richtext.statistics.characters", "character", "characters"],
] as const;

describe("desktop count messages", () => {
  const i18n = createI18n({ legacy: false, locale: "en", messages: localeMessages });

  it.each(examples)("selects the full singular and plural message for %s", (key, singular, plural) => {
    expect(i18n.global.t(key, { count: 1 })).toBe(`1 ${singular}`);
    expect(i18n.global.t(key, { count: 2 })).toBe(`2 ${plural}`);
    expect(i18n.global.t(key, { count: 0 })).toBe(`0 ${plural}`);
  });
  it("keeps archived deletion and subset counts grammatical at one", () => {
    expect(i18n.global.t("settings.archivedSpaces.deleteAll.description", { count: 1 })).toBe(
      "Permanently delete 1 archived Space and its contents? This cannot be undone.",
    );
    expect(i18n.global.t("settings.archivedSpaces.deleteAll.description", { count: 2 })).toBe(
      "Permanently delete 2 archived Spaces and their contents? This cannot be undone.",
    );
    expect(i18n.global.t("spaces.processing.document.textCoverage", { count: 1, total: 1 })).toBe(
      "Pages with extracted text: 1 of 1.",
    );
    expect(i18n.global.t("spaces.processing.document.visualCoverage", { count: 1, total: 1 })).toBe(
      "Sources analyzed: 1 of 1.",
    );
  });
});

describe("Russian count messages", () => {
  it.each([
    [0, "0 моделей"],
    [1, "1 модель"],
    [2, "2 модели"],
    [5, "5 моделей"],
    [11, "11 моделей"],
    [12, "12 моделей"],
    [14, "14 моделей"],
    [21, "21 модель"],
    [22, "22 модели"],
    [25, "25 моделей"],
    [101, "101 модель"],
    [111, "111 моделей"],
  ])("selects the correct form for %s", (count, expected) => {
    const previous = appI18n.global.locale.value;
    appI18n.global.locale.value = "ru";
    try {
      expect(appI18n.global.t("settings.ai.models.count", { count }, count)).toBe(expected);
    } finally {
      appI18n.global.locale.value = previous;
    }
  });
  it.each([
    [1, "1 слово", "1 символ"],
    [2, "2 слова", "2 символа"],
    [5, "5 слов", "5 символов"],
  ])("inflects editor statistics at %s", (count, words, characters) => {
    const previous = appI18n.global.locale.value;
    appI18n.global.locale.value = "ru";
    try {
      expect(appI18n.global.t("spaces.richtext.statistics.words", { count }, count)).toBe(words);
      expect(appI18n.global.t("spaces.richtext.statistics.characters", { count }, count)).toBe(characters);
    } finally {
      appI18n.global.locale.value = previous;
    }
  });
});
