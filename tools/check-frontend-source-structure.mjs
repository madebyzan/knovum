import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { checkFrontendSourceNaming } from "./frontend-source-naming.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const sourceRoot = join(root, "src");
const sourceFiles = filesUnder(sourceRoot);
const failures = [
  ...checkFrontendSourceNaming(root, sourceRoot, sourceFiles),
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
