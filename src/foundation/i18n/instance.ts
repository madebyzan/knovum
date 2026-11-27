import { createI18n } from "vue-i18n";

import {
  enMessages,
  loadLocaleMessages,
  SUPPORTED_LOCALES,
  type Locale,
  type Messages,
} from "./locales/locale-loader";

export type AppLocale = Locale;

const supportedLocaleSet = new Set<string>(SUPPORTED_LOCALES);
const loadedLocales = new Set<AppLocale>(["en"]);
const russianPlurals = new Intl.PluralRules("ru");

const COMPACT_NUMBER_FORMAT = { notation: "compact", maximumFractionDigits: 1 } as const;
const DATE_TIME_FORMATS = {
  short: { dateStyle: "short", timeStyle: "short" },
  medium: { dateStyle: "medium" },
} as const;

export const i18n = createI18n({
  legacy: false,
  locale: "en" as AppLocale,
  fallbackLocale: "en",
  messages: { en: enMessages } as Record<AppLocale, Messages>,
  numberFormats: Object.fromEntries(
    SUPPORTED_LOCALES.map((locale) => [locale, { compact: COMPACT_NUMBER_FORMAT }]),
  ),
  datetimeFormats: Object.fromEntries(SUPPORTED_LOCALES.map((locale) => [locale, DATE_TIME_FORMATS])),
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

export async function ensureLocaleLoaded(locale: AppLocale): Promise<void> {
  if (loadedLocales.has(locale)) return;
  const messages = await loadLocaleMessages(locale);
  i18n.global.setLocaleMessage(locale, messages);
  loadedLocales.add(locale);
}

export function systemLocale(locale: string | null): AppLocale {
  if (locale === null) return "en";
  const { language, script, region } = new Intl.Locale(locale);
  if (language === "zh") {
    if (script === "Hans") return "zh-CN";
    return script === "Hant" || ["TW", "HK", "MO"].includes(region ?? "") ? "zh-TW" : "zh-CN";
  }
  if (language === "pt") return "pt-BR";
  return supportedLocaleSet.has(language) ? (language as AppLocale) : "en";
}
