import { camelCase } from "es-toolkit";
import { isPlainObject } from "es-toolkit/predicate";
import type { LocaleMessageValue } from "vue-i18n";

import type { LocalePreference } from "../../../shared/contracts/generated/LocalePreference";

export type Locale = Exclude<LocalePreference, "system">;
export type Messages = Record<string, LocaleMessageValue>;

export const SUPPORTED_LOCALES = [
  "en",
  "zh-CN",
  "zh-TW",
  "ja",
  "ko",
  "fr",
  "de",
  "es",
  "pt-BR",
  "it",
  "ru",
] as const satisfies readonly Locale[];

const enModules = import.meta.glob<{ default: Messages }>("./en/**/*.json", { eager: true });
const lazyModules = import.meta.glob<{ default: Messages }>(["./*/**/*.json", "!./en/**/*.json"]);

export function assembleLocaleMessages(
  locale: Locale,
  entries: Iterable<readonly [string, Messages]>,
): Messages {
  const root: Messages = {};
  const prefix = `./${locale}/`;

  for (const [file, messages] of entries) {
    if (!file.startsWith(prefix)) continue;

    const keys = file
      .slice(prefix.length, -5)
      .split("/")
      .map((seg) => camelCase(seg));
    let node = root;

    for (const key of keys.slice(0, -1)) {
      const value = node[key];

      if (value === undefined) {
        node[key] = {};
      } else if (!isPlainObject(value)) {
        throw new Error(`Locale path conflict: ${file}`);
      }

      node = node[key] as Messages;
    }

    node[keys.at(-1)!] = messages;
  }

  return root;
}

export const enMessages: Messages = assembleLocaleMessages(
  "en",
  Object.entries(enModules).map(([file, mod]) => [file, mod.default] as const),
);

const pendingByLocale = new Map<Locale, Promise<Messages>>();

export function loadLocaleMessages(locale: Locale): Promise<Messages> {
  if (locale === "en") return Promise.resolve(enMessages);
  const existing = pendingByLocale.get(locale);
  if (existing) return existing;
  const prefix = `./${locale}/`;
  const matching = Object.entries(lazyModules).filter(([file]) => file.startsWith(prefix));
  const loading = Promise.all(
    matching.map(async ([file, loader]) => [file, (await loader()).default] as const),
  ).then((entries) => assembleLocaleMessages(locale, entries));
  pendingByLocale.set(locale, loading);
  loading.catch(() => pendingByLocale.delete(locale));
  return loading;
}
