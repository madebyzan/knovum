import { createI18n } from "vue-i18n";

import { localeMessages } from "../../locales/locale-registry";

export type AppLocale = "en" | "zh-CN";

const compact = { notation: "compact", maximumFractionDigits: 1 } as const;
const dateTimeFormats = {
  short: { dateStyle: "short", timeStyle: "short" },
  medium: { dateStyle: "medium" },
} as const;

export const i18n = createI18n({
  legacy: false,
  locale: systemLocale(globalThis.navigator.language),
  fallbackLocale: false,
  messages: localeMessages,
  numberFormats: {
    en: { compact },
    "zh-CN": { compact },
  },
  dateTimeFormats: {
    en: dateTimeFormats,
    "zh-CN": dateTimeFormats,
  },
});

export function systemLocale(locale: string | null): AppLocale {
  return locale?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}
