import { describe, expect, it } from "vitest";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../locales/locale-registry";

const examples = [
  ["settings.ai.modelCount", "model", "models"],
  ["spaces.inspector.itemsSelected", "item selected", "items selected"],
  ["spaces.trash.structure.deletedCount", "deleted node", "deleted nodes"],
  ["spaces.space.nodeCount", "item", "items"],
] as const;

describe("desktop count messages", () => {
  const i18n = createI18n({ legacy: false, locale: "en", messages: localeMessages });

  it.each(examples)("selects the full singular and plural message for %s", (key, singular, plural) => {
    expect(i18n.global.t(key, { count: 1 })).toBe(`1 ${singular}`);
    expect(i18n.global.t(key, { count: 2 })).toBe(`2 ${plural}`);
    expect(i18n.global.t(key, { count: 0 })).toBe(`0 ${plural}`);
  });
  it("keeps archived deletion and subset counts grammatical at one", () => {
    expect(i18n.global.t("settings.general.archived.deleteAll.description", { count: 1 })).toBe(
      "Permanently delete 1 archived Space and its contents? This cannot be undone.",
    );
    expect(i18n.global.t("settings.general.archived.deleteAll.description", { count: 2 })).toBe(
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
