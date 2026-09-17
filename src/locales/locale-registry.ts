import { camelCase } from "es-toolkit";
import { isPlainObject } from "es-toolkit/predicate";
import type { LocaleMessageValue } from "vue-i18n";

type Locale = "en" | "zh-cn";
type Messages = Record<string, LocaleMessageValue>;

const modules = import.meta.glob<{ default: Messages }>("./{en,zh-cn}/**/*.json", { eager: true });

function createLocaleMessages(locale: Locale): Messages {
  const root: Messages = {};
  const prefix = `./${locale}/`;

  for (const [file, module] of Object.entries(modules)) {
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

    node[keys.at(-1)!] = module.default;
  }

  return root;
}

export const localeMessages = {
  en: createLocaleMessages("en"),
  "zh-CN": createLocaleMessages("zh-cn"),
} as const;
