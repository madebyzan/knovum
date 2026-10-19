import { describe, expect, it, vi } from "vitest";

import { i18n, type AppLocale } from "../instance";
import { localeMessages } from "../locales/locale-registry";

describe("i18n datetime formats", () => {
  it("formats dates with short and medium formats across all locales without warnings", () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const date = new Date("2026-10-19T12:00:00Z");
    const locales = Object.keys(localeMessages) as AppLocale[];

    try {
      for (const locale of locales) {
        i18n.global.locale.value = locale;
        const shortFormatted = i18n.global.d(date, "short");
        const mediumFormatted = i18n.global.d(date, "medium");

        expect(shortFormatted).toBeTruthy();
        expect(mediumFormatted).toBeTruthy();
      }
      expect(warnSpy).not.toHaveBeenCalled();
    } finally {
      warnSpy.mockRestore();
    }
  });
});
