import { createI18n } from "vue-i18n";

import { localeMessages } from "../../locales/locale-registry";

export type AppLocale = "en" | "zh-CN";

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
  numberFormats: {
    en: { compact: COMPACT_NUMBER_FORMAT },
    "zh-CN": { compact: COMPACT_NUMBER_FORMAT },
  },
  dateTimeFormats: {
    en: DATE_TIME_FORMATS,
    "zh-CN": DATE_TIME_FORMATS,
  },
});

export function systemLocale(locale: string | null): AppLocale {
  return locale?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}
