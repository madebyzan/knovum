import { difference } from "es-toolkit";
import { describe, expect, it } from "vitest";

import { localeMessages } from "../../../locales/locale-registry";

type LocaleTree = { readonly [key: string]: LocaleTree | string };

const sourceModules = import.meta.glob("/src/**/*.{ts,vue}", {
  eager: true,
  import: "default",
  query: "?raw",
}) as Record<string, string>;

const localeFragments = import.meta.glob("/src/locales/{en,zh-cn}/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, Record<string, unknown>>;

// These namespaces key their messages by a serialized domain value, so the key carries that value's
// own casing rather than the naming convention every other message key follows.
const serializedValueMaps = new Set([
  "settings.ai.agentConnections",
  "spaces.assistant.aiScope",
  "spaces.knowledge.knowledgeFieldKindLabel",
  "spaces.knowledge.knowledgeHealthIssue",
  "spaces.knowledge.relation",
  "spaces.processing.enrichmentJobType",
  "spaces.processing.documentWarnings",
]);

function flattenLocale(tree: LocaleTree, parent = "", keys = new Set<string>()) {
  for (const [name, value] of Object.entries(tree)) {
    const key = parent ? `${parent}.${name}` : name;
    if (typeof value === "string") keys.add(key);
    else flattenLocale(value, key, keys);
  }
  return keys;
}

function flattenMessages(tree: LocaleTree, parent = "", messages = new Map<string, string>()) {
  for (const [name, value] of Object.entries(tree)) {
    const key = parent ? `${parent}.${name}` : name;
    if (typeof value === "string") messages.set(key, value);
    else flattenMessages(value, key, messages);
  }
  return messages;
}

function placeholders(message: string) {
  return [...message.matchAll(/\{([^}]+)\}/g)].map((match) => match[1]).toSorted();
}

function localeModulePath(path: string) {
  return path.replace(/^.*\/locales\/(?:en|zh-cn)\//u, "");
}

function nonCamelCaseKeys(tree: Record<string, unknown>, parent = "", keys: string[] = []) {
  for (const [key, value] of Object.entries(tree)) {
    const path = parent ? `${parent}.${key}` : key;
    if (!serializedValueMaps.has(parent) && !/^[a-z][A-Za-z0-9]*$/u.test(key)) keys.push(path);
    if (value && typeof value === "object") {
      nonCamelCaseKeys(value as Record<string, unknown>, path, keys);
    }
  }
  return keys;
}

function collectStaticTranslationKeys() {
  const keys = new Set<string>();
  const translationCall = /(?:\$t|\bt)\(\s*["'`]([^"'`$]+)["'`]/g;
  for (const [path, source] of Object.entries(sourceModules)) {
    if (path.includes("/locales/") || path.endsWith("locale-completeness.test.ts")) continue;
    for (const match of source.matchAll(translationCall)) keys.add(match[1]!);
  }
  return keys;
}

describe("locale completeness", () => {
  const { en, "zh-CN": zhCN } = localeMessages;
  const enKeys = flattenLocale(en as LocaleTree);
  const zhCNKeys = flattenLocale(zhCN as LocaleTree);

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
    const english = flattenMessages(en as LocaleTree);
    const chinese = flattenMessages(zhCN as LocaleTree);
    const mismatches = [...english].flatMap(([key, message]) => {
      const expected = placeholders(message);
      const actual = placeholders(chinese.get(key) ?? "");
      return expected.join() === actual.join() ? [] : [{ key, expected, actual }];
    });
    expect(mismatches).toEqual([]);
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
});
