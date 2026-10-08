import { camelCase, isPlainObject } from "es-toolkit";
import { describe, expect, it } from "vitest";

const localeFragments = import.meta.glob("/src/locales/{en,zh-cn}/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, unknown>;

interface ModulePath {
  locale: string;
  segments: string[];
}

interface ValueIssues {
  blankMessages: string[];
  emptyGroups: string[];
  nonStringLeaves: string[];
}

function parseModulePath(path: string): ModulePath {
  const match = path.match(/\/locales\/(en|zh-cn)\/(.+)\.json$/u);
  if (!match) throw new Error(`Unexpected locale module path: ${path}`);
  return { locale: match[1]!, segments: match[2]!.split("/") };
}

function displayModulePath(path: string) {
  const { locale, segments } = parseModulePath(path);
  return `${locale}/${segments.join("/")}.json`;
}

function namespaceCollisions() {
  const claims = new Map<string, Set<string>>();

  for (const path of Object.keys(localeFragments)) {
    const { locale, segments } = parseModulePath(path);
    for (const [index] of segments.entries()) {
      const namespace = segments
        .slice(0, index + 1)
        .map((segment) => camelCase(segment))
        .join(".");
      const kind = index === segments.length - 1 ? "file" : "directory";
      const diskPath = segments.slice(0, index + 1).join("/");
      const key = `${locale}:${namespace}`;
      const namespaceClaims = claims.get(key) ?? new Set<string>();
      namespaceClaims.add(`${diskPath} (${kind})`);
      claims.set(key, namespaceClaims);
    }
  }

  return [...claims].flatMap(([namespace, namespaceClaims]) =>
    namespaceClaims.size > 1 ? [`${namespace}: ${[...namespaceClaims].join(", ")}`] : [],
  );
}

function inspectValue(value: unknown, path: string, issues: ValueIssues) {
  if (typeof value === "string") {
    if (value.trim() === "") issues.blankMessages.push(path);
    return;
  }
  if (!isPlainObject(value)) {
    issues.nonStringLeaves.push(path);
    return;
  }

  const entries = Object.entries(value);
  if (entries.length === 0) issues.emptyGroups.push(path);
  for (const [key, child] of entries) inspectValue(child, `${path}.${key}`, issues);
}

function collectValueIssues(): ValueIssues {
  const issues: ValueIssues = { blankMessages: [], emptyGroups: [], nonStringLeaves: [] };
  for (const [path, fragment] of Object.entries(localeFragments)) {
    inspectValue(fragment, displayModulePath(path), issues);
  }
  return issues;
}

describe("locale structure", () => {
  it("uses kebab-case segments in every locale module path", () => {
    const invalidPaths = Object.keys(localeFragments)
      .filter((path) =>
        parseModulePath(path).segments.some((segment) => !/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(segment)),
      )
      .map(displayModulePath);
    expect(invalidPaths).toEqual([]);
  });

  it("keeps normalized locale namespaces collision-free", () => {
    expect(namespaceCollisions()).toEqual([]);
  });

  it("contains no empty message groups", () => {
    expect(collectValueIssues().emptyGroups).toEqual([]);
  });

  it("contains no blank messages", () => {
    expect(collectValueIssues().blankMessages).toEqual([]);
  });

  it("contains only string message leaves", () => {
    expect(collectValueIssues().nonStringLeaves).toEqual([]);
  });
});
