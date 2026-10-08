import { difference, isPlainObject } from "es-toolkit";
import { describe, expect, it } from "vitest";

import { dynamicLocaleKeyPrefixes, dynamicLocaleKeys } from "../../../locales/dynamic-keys";
import { localeMessages } from "../../../locales/locale-registry";

type LocaleTree = { readonly [key: string]: LocaleTree | string };

const sourceModules = import.meta.glob(
  [
    "/src/**/*.{ts,vue}",
    "!/src/locales/**",
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

const localeFragments = import.meta.glob("/src/locales/{en,zh-cn}/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, Record<string, unknown>>;

// These namespaces key their messages by a serialized domain value, so the key carries that value's
// own casing rather than the naming convention every other message key follows.
const serializedValueMaps = new Set([
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
  return path.replace(/^.*\/locales\/(?:en|zh-cn)\//u, "");
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
  const { en, "zh-CN": zhCN } = localeMessages;
  const english = flattenMessages(en as LocaleTree);
  const chinese = flattenMessages(zhCN as LocaleTree);
  const enKeys = new Set(english.keys());
  const zhCNKeys = new Set(chinese.keys());

  it("keeps English and Simplified Chinese keys aligned", () => {
    expect(difference([...enKeys], [...zhCNKeys])).toEqual([]);
    expect(difference([...zhCNKeys], [...enKeys])).toEqual([]);
  });

  it("defines every statically referenced translation key", () => {
    const usedKeys = collectStaticTranslationKeys();
    expect(difference([...usedKeys], [...enKeys])).toEqual([]);
    expect(difference([...usedKeys], [...zhCNKeys])).toEqual([]);
  });

  it("keeps placeholders aligned between languages", () => {
    const mismatches = [...english].flatMap(([key, message]) => {
      const expected = placeholders(message);
      const actual = placeholders(chinese.get(key) ?? "");
      return expected.join() === actual.join() ? [] : [{ key, expected, actual }];
    });
    expect(mismatches).toEqual([]);
  });

  it("treats repeated placeholders as one named argument", () => {
    expect(placeholders("{count} item | {count} items")).toEqual(["count"]);
  });

  it("keeps locale module paths aligned", () => {
    const paths = Object.keys(localeFragments);
    const enPaths = paths.filter((path) => path.includes("/locales/en/")).map(localeModulePath);
    const zhCNPaths = paths.filter((path) => path.includes("/locales/zh-cn/")).map(localeModulePath);
    expect(difference(enPaths, zhCNPaths)).toEqual([]);
    expect(difference(zhCNPaths, enPaths)).toEqual([]);
  });

  it("uses semantic lowerCamelCase message keys", () => {
    expect(nonCamelCaseKeys(en as unknown as Record<string, unknown>)).toEqual([]);
    expect(nonCamelCaseKeys(zhCN as unknown as Record<string, unknown>)).toEqual([]);
  });

  it("keeps every message key reachable from product code", () => {
    const referenced = collectReferencedKeyLiterals();
    const runtimeResolved: readonly string[] = dynamicLocaleKeys;
    const unreachable = [...enKeys].filter(
      (key) =>
        !referenced.has(key) &&
        !runtimeResolved.includes(key) &&
        !dynamicLocaleKeyPrefixes.some((prefix) => key.startsWith(prefix)),
    );
    expect(unreachable).toEqual([]);
  });

  it("keeps the runtime-resolved key allowlist narrowed to real keys", () => {
    const stalePrefixes = dynamicLocaleKeyPrefixes.filter(
      (prefix) => ![...enKeys].some((key) => key.startsWith(prefix)),
    );
    const staleKeys = dynamicLocaleKeys.filter((key) => !enKeys.has(key));
    expect([...stalePrefixes, ...staleKeys]).toEqual([]);
  });
});
