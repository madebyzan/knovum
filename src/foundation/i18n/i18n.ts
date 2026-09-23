import { createI18n } from "vue-i18n";

import { localeMessages } from "../../locales/locale-registry";

export type AppLocale = "en" | "zh-CN";

export const i18n = createI18n({
  legacy: false,
  locale: systemLocale(globalThis.navigator.language),
  fallbackLocale: false,
  messages: localeMessages,
  numberFormats: {
    en: { compact: { notation: "compact", maximumFractionDigits: 1 } },
    "zh-CN": { compact: { notation: "compact", maximumFractionDigits: 1 } },
  },
});

export function systemLocale(locale: string | null): AppLocale {
  return locale?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}
