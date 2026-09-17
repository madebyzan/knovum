import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { checkFrontendSourceNaming } from "./frontend-source-naming.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const sourceRoot = join(root, "src");
const MAX_FILE_LINES = 180;

const sourceFiles = filesUnder(sourceRoot);
const failures = sourceFiles
  .map((file) => ({ file, lines: effectiveLines(readFileSync(file, "utf8")) }))
  .filter(({ lines }) => lines > MAX_FILE_LINES)
  .map(({ file, lines }) => `${relative(root, file)}: ${lines} effective lines exceeds ${MAX_FILE_LINES}`);
failures.push(...checkFrontendSourceNaming(root, sourceRoot, sourceFiles));
failures.push(...checkRetiredArchitecturePaths(root));

if (failures.length) {
  console.error(
    `Frontend source structure gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}

console.log("Frontend source structure gate: file size and naming rules are clean");

function filesUnder(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path));
    else if (entry.isFile() && (path.endsWith(".ts") || path.endsWith(".vue"))) files.push(path);
  }
  return files;
}

function effectiveLines(source) {
  return source.split(/\r?\n/u).filter((line) => line.trim().length > 0).length;
}

function checkRetiredArchitecturePaths(repositoryRoot) {
  const retiredPaths = [
    "src/foundation/integrations",
    "src/features/spaces/components",
    "src/features/spaces/navigation",
    "src/features/spaces/application",
  ];
  return retiredPaths
    .filter((path) => existsSync(join(repositoryRoot, path)))
    .map((path) => `${path}: retired architecture path must not be recreated`);
}
