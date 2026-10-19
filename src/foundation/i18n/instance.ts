import { createI18n } from "vue-i18n";

import { localeMessages } from "./locales/locale-registry";

export type AppLocale = keyof typeof localeMessages;

const locales = Object.keys(localeMessages);
const russianPlurals = new Intl.PluralRules("ru");

const COMPACT_NUMBER_FORMAT = { notation: "compact", maximumFractionDigits: 1 } as const;
const DATE_TIME_FORMATS = {
  short: { dateStyle: "short", timeStyle: "short" },
  medium: { dateStyle: "medium" },
} as const;

export const i18n = createI18n({
  legacy: false,
  locale: systemLocale(globalThis.navigator.language),
  fallbackLocale: false,
  messages: localeMessages,
  numberFormats: Object.fromEntries(locales.map((locale) => [locale, { compact: COMPACT_NUMBER_FORMAT }])),
  datetimeFormats: Object.fromEntries(locales.map((locale) => [locale, DATE_TIME_FORMATS])),
  pluralRules: {
    fr: (count) => (count === 0 || count === 1 ? 0 : 1),
    "pt-BR": (count) => (count === 0 || count === 1 ? 0 : 1),
    ru: (count, length) => {
      if (length === 1) return 0;
      const category = russianPlurals.select(count);
      if (category === "one") return 0;
      return category === "many" ? 2 : 1;
    },
  },
});

export function systemLocale(locale: string | null): AppLocale {
  if (locale === null) return "en";
  const { language, script, region } = new Intl.Locale(locale);
  if (language === "zh") {
    if (script === "Hans") return "zh-CN";
    return script === "Hant" || ["TW", "HK", "MO"].includes(region ?? "") ? "zh-TW" : "zh-CN";
  }
  if (language === "pt") return "pt-BR";
  return Object.hasOwn(localeMessages, language) ? (language as AppLocale) : "en";
}
