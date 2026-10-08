import { readFileSync, readdirSync } from "node:fs";
import { basename, dirname, extname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

export const sourceRoots = [
  "src",
  "src-tauri/src",
  "src-tauri/tests",
  "src-tauri/build_support",
  "scripts",
  "python/src",
  "python/tests",
  "tests",
  "docs",
];
const sourceExtensions = new Set([".rs", ".ts", ".vue", ".mjs", ".py"]);
const testDirectories = new Set(["tests", "__tests__", "e2e"]);
const generatedDirectories = new Set(["__pycache__"]);

function inspectPathIdentity(path, kind) {
  const name = basename(path);
  if (/^native(?:[-_.]|$)/iu.test(name) || /^[Nn]ative[A-Z0-9]/u.test(name)) {
    return [`${path}: native is not a path identity`];
  }
  if (kind === "file" && name.toLowerCase() === "index.ts")
    return [`${path}: TypeScript index modules are forbidden`];
  return [];
}

export function inspectSourceLayout(root, recursive = true) {
  const violations = [];
  const entries = readdirSync(root, { recursive, withFileTypes: true });
  const directories = new Set(
    entries
      .filter((entry) => entry.isDirectory())
      .map((entry) => join(entry.parentPath, entry.name).toLowerCase()),
  );
  for (const entry of entries) {
    if (entry.isDirectory())
      violations.push(...inspectPathIdentity(join(entry.parentPath, entry.name), "directory"));
  }
  for (const entry of entries) {
    if (!entry.isFile()) continue;
    const file = join(entry.parentPath, entry.name);
    const extension = extname(file);
    const directory = join(dirname(file), basename(file, extension));
    if (directories.has(directory.toLowerCase())) {
      violations.push(`${file}: conflicts with directory ${directory}`);
    }
    if (!file.split(sep).some((part) => generatedDirectories.has(part))) {
      violations.push(...inspectPathIdentity(file, "file"));
    }
    if (!sourceExtensions.has(extension)) continue;
    violations.push(...inspectSourceFile(file, extension));
  }
  return violations;
}

function inspectSourceFile(file, extension) {
  const violations = [];
  const inTests = file.split(sep).some((part) => testDirectories.has(part));
  const source = readFileSync(file, "utf8");
  const testName = /(?:^tests?\.|^test_|_tests\.|\.(?:test|spec)\.)/.test(basename(file));
  const testCode =
    extension === ".rs"
      ? /^\s*#\[(?:tokio::)?test(?:\]|\()/m.test(source)
      : extension === ".py"
        ? /^\s*(?:async )?def test_/m.test(source)
        : /\bfrom\s*["'](?:vitest|@vue\/test-utils|node:test)["']/.test(source);
  if (!inTests && (testName || testCode)) {
    violations.push(`${file}: test code must live in a tests/ or __tests__/ directory`);
  }
  if ([".rs", ".py", ".vue"].includes(extension)) {
    const lines = source.split("\n").filter((line) => line.trim()).length;
    const limit = extension === ".vue" ? 180 : 350;
    if (lines > limit)
      violations.push(`${file}: ${lines} nonblank lines exceeds the ${limit}-line source limit`);
  }
  return violations;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const violations = [
    ...[".", "python", "src-tauri"].flatMap((root) => inspectSourceLayout(root, false)),
    ...sourceRoots.flatMap((root) => inspectSourceLayout(root)),
  ];
  if (violations.length) {
    console.error(violations.join("\n"));
    process.exitCode = 1;
  } else {
    console.log("Source layout: path identities, test placement and source size meet policy.");
  }
}
