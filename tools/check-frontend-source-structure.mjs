import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { checkFrontendSourceNaming } from "./frontend-source-naming.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const sourceRoot = join(root, "src");
const sourceFiles = filesUnder(sourceRoot);
const failures = [
  ...checkFrontendSourceNaming(root, sourceRoot, sourceFiles),
  ...checkRootIntegrationTestOwnership(root, sourceFiles),
  ...checkRetiredArchitecturePaths(root),
];

if (failures.length) {
  console.error(
    `Frontend source structure gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}

console.log("Frontend source structure gate: naming, test ownership and retired paths are clean");

function filesUnder(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path));
    else if (entry.isFile() && (path.endsWith(".ts") || path.endsWith(".vue"))) files.push(path);
  }
  return files;
}

function checkRootIntegrationTestOwnership(repositoryRoot, files) {
  const spacesRoot = join(repositoryRoot, "src", "features", "spaces");
  const rootTests = join(spacesRoot, "__tests__");
  const sourcePaths = new Set(files);
  const failures = [];

  for (const file of files) {
    if (dirname(file) !== rootTests || !file.endsWith(".test.ts")) continue;
    const capabilities = new Set();
    const source = readFileSync(file, "utf8");
    for (const match of source.matchAll(
      /(?:from\s+|import\s*\(\s*|vi\.(?:mock|doMock|unmock)\(\s*)["'](\.\.?\/[^"']+)["']/gu,
    )) {
      const target = resolveSourceImport(file, match[1], sourcePaths);
      if (!target) continue;
      const [capability] = relative(spacesRoot, target).split(sep);
      if (capability && capability !== "__tests__" && capability !== "testing") {
        capabilities.add(capability);
      }
    }
    if (capabilities.size === 1) {
      const [owner] = capabilities;
      failures.push(
        `${relative(repositoryRoot, file)}: single-capability test belongs under the ${owner} owner, not the Spaces integration-test root`,
      );
    }
  }
  return failures;
}

function resolveSourceImport(file, specifier, sourcePaths) {
  const base = resolve(dirname(file), specifier);
  for (const candidate of [base, `${base}.ts`, `${base}.vue`]) {
    if (sourcePaths.has(candidate)) return candidate;
  }
  return undefined;
}

function checkRetiredArchitecturePaths(repositoryRoot) {
  const retiredPaths = [
    "src/foundation/integrations",
    "src/features/spaces/components",
    "src/features/spaces/navigation",
    "src/features/spaces/application",
    "src/features/spaces/session/actions",
  ];
  return retiredPaths
    .filter((path) => existsSync(join(repositoryRoot, path)))
    .map((path) => `${path}: retired architecture path must not be recreated`);
}
