import { difference, isPlainObject } from "es-toolkit";
import { describe, expect, it } from "vitest";

import { preferenceLocales } from "../../../shared/contracts/generated/contract-values";
import { DYNAMIC_LOCALE_KEY_PREFIXES, DYNAMIC_LOCALE_KEYS } from "../locales/dynamic-keys";
import { localeMessages } from "../locales/locale-registry";

type LocaleTree = { readonly [key: string]: LocaleTree | string };

const sourceModules = import.meta.glob(
  [
    "/src/**/*.{ts,vue}",
    "!/src/foundation/i18n/locales/**",
    "!/src/**/__tests__/**",
    "!/src/**/tests/**",
    "!/src/**/*.test.ts",
    "!/src/**/*.spec.ts",
  ],
  {
    eager: true,
    import: "default",
    query: "?raw",
  },
) as Record<string, string>;

const localeFragments = import.meta.glob("/src/foundation/i18n/locales/*/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, Record<string, unknown>>;

// These namespaces key their messages by a serialized domain value, so the key carries that value's
// own casing rather than the naming convention every other message key follows.
const serializedValueMaps = new Set([
  "extensions.bundled",
  "spaces.knowledge.fields.kindLabels",
  "spaces.knowledge.health.issues",
  "spaces.knowledge.relations.kinds",
  "spaces.processing.enrichmentJobType",
  "spaces.processing.jobStatus",
]);

function flattenMessages(tree: LocaleTree, parent = "", messages = new Map<string, string>()) {
  for (const [name, value] of Object.entries(tree)) {
    const key = parent ? `${parent}.${name}` : name;
    if (typeof value === "string") messages.set(key, value);
    else flattenMessages(value, key, messages);
  }
  return messages;
}

function placeholders(message: string) {
  return [...new Set([...message.matchAll(/\{([^}]+)\}/g)].map((match) => match[1]!))].toSorted();
}

function localeModulePath(path: string) {
  return path.replace(/^.*\/locales\/[^/]+\//u, "");
}

function nonCamelCaseKeys(tree: Record<string, unknown>, parent = "", keys: string[] = []) {
  for (const [key, value] of Object.entries(tree)) {
    const path = parent ? `${parent}.${key}` : key;
    if (!serializedValueMaps.has(parent) && !/^[a-z][A-Za-z0-9]*$/u.test(key)) keys.push(path);
    if (isPlainObject(value)) {
      nonCamelCaseKeys(value, path, keys);
    }
  }
  return keys;
}

function collectStaticTranslationKeys() {
  const keys = new Set<string>();
  const translationCall = /(?:\$t|\bt)\(\s*["'`]([^"'`$]+)["'`]/g;
  for (const source of Object.values(sourceModules)) {
    for (const match of source.matchAll(translationCall)) keys.add(match[1]!);
  }
  return keys;
}

// Any dotted literal naming a locale key, which also covers keys held in a `labelKey`-style map
// and translated later through a variable.
function collectReferencedKeyLiterals() {
  const namespaces = Object.keys(localeMessages.en).join("|");
  const literal = new RegExp(`["'\`]((?:${namespaces})(?:\\.[A-Za-z0-9_-]+)*\\.?)["'\`]`, "g");
  const keys = new Set<string>();
  for (const source of Object.values(sourceModules)) {
    for (const match of source.matchAll(literal)) keys.add(match[1]!);
  }
  return keys;
}

describe("locale completeness", () => {
  const english = flattenMessages(localeMessages.en as LocaleTree);
  const enKeys = new Set(english.keys());
  const locales = Object.entries(localeMessages).map(
    ([locale, tree]) => [locale, flattenMessages(tree as LocaleTree)] as const,
  );

  it("loads exactly the supported language roots", () => {
    const expected = preferenceLocales.filter((locale) => locale !== "system").toSorted();
    const roots = [
      ...new Set(Object.keys(localeFragments).map((path) => path.split("/locales/")[1]!.split("/")[0]!)),
    ].toSorted();
    expect(roots).toEqual(expected);
    expect(Object.keys(localeMessages).toSorted()).toEqual(expected);
  });

  it.each(locales)("keeps %s keys aligned with English", (_locale, messages) => {
    expect(difference([...enKeys], [...messages.keys()])).toEqual([]);
    expect(difference([...messages.keys()], [...enKeys])).toEqual([]);
  });

  it.each(locales)("defines all product references in %s", (_locale, messages) => {
    expect(difference([...collectStaticTranslationKeys()], [...messages.keys()])).toEqual([]);
  });

  it.each(locales)("keeps %s placeholders aligned with English", (_locale, messages) => {
    const mismatches = [...english].flatMap(([key, message]) => {
      const expected = placeholders(message);
      const actual = placeholders(messages.get(key) ?? "");
      return expected.join() === actual.join() ? [] : [{ key, expected, actual }];
    });
    expect(mismatches).toEqual([]);
  });

  it("treats repeated placeholders as one named argument", () => {
    expect(placeholders("{count} item | {count} items")).toEqual(["count"]);
  });

  it.each(locales)("keeps %s module paths aligned with English", (locale) => {
    const paths = Object.keys(localeFragments);
    const enPaths = paths.filter((path) => path.includes("/locales/en/")).map(localeModulePath);
    const localizedPaths = paths.filter((path) => path.includes(`/locales/${locale}/`)).map(localeModulePath);
    expect(difference(enPaths, localizedPaths)).toEqual([]);
    expect(difference(localizedPaths, enPaths)).toEqual([]);
  });

  it.each(Object.entries(localeMessages))("uses lowerCamelCase keys in %s", (_locale, tree) => {
    expect(nonCamelCaseKeys(tree as unknown as Record<string, unknown>)).toEqual([]);
  });

  it("keeps every message key reachable from product code", () => {
    const referenced = collectReferencedKeyLiterals();
    const runtimeResolved: readonly string[] = DYNAMIC_LOCALE_KEYS;
    const unreachable = [...enKeys].filter(
      (key) =>
        !referenced.has(key) &&
        !runtimeResolved.includes(key) &&
        !DYNAMIC_LOCALE_KEY_PREFIXES.some((prefix) => key.startsWith(prefix)),
    );
    expect(unreachable).toEqual([]);
  });

  it("keeps the runtime-resolved key allowlist narrowed to real keys", () => {
    const stalePrefixes = DYNAMIC_LOCALE_KEY_PREFIXES.filter(
      (prefix) => ![...enKeys].some((key) => key.startsWith(prefix)),
    );
    const staleKeys = DYNAMIC_LOCALE_KEYS.filter((key) => !enKeys.has(key));
    expect([...stalePrefixes, ...staleKeys]).toEqual([]);
  });
});
