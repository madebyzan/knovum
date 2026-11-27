import { i18n, type AppLocale } from "./instance";
import { localeMessages } from "./locales/locale-registry";

// Preload all locales in Vitest so synchronous locale-switching tests work without async loaders.
for (const [locale, messages] of Object.entries(localeMessages)) {
  i18n.global.setLocaleMessage(locale as AppLocale, messages);
}
