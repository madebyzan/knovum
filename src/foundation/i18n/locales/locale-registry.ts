import { assembleLocaleMessages, type Locale, type Messages } from "./locale-loader";

const modules = import.meta.glob<{ default: Messages }>("./*/**/*.json", { eager: true });
const entries = Object.entries(modules).map(([file, mod]) => [file, mod.default] as const);

function createLocaleMessages(locale: Locale): Messages {
  return assembleLocaleMessages(locale, entries);
}

export const localeMessages = {
  en: createLocaleMessages("en"),
  "zh-CN": createLocaleMessages("zh-CN"),
  "zh-TW": createLocaleMessages("zh-TW"),
  ja: createLocaleMessages("ja"),
  ko: createLocaleMessages("ko"),
  fr: createLocaleMessages("fr"),
  de: createLocaleMessages("de"),
  es: createLocaleMessages("es"),
  "pt-BR": createLocaleMessages("pt-BR"),
  it: createLocaleMessages("it"),
  ru: createLocaleMessages("ru"),
} satisfies Record<Locale, Messages>;
