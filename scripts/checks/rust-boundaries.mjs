import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const CONCRETE_RUN_CAPABILITIES = [
  "conversations",
  "derivation",
  "external_conversations",
  "speech",
  "workflows",
];

function rustFiles(directory) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) return entry === "tests" ? [] : rustFiles(path);
    return path.endsWith(".rs") ? [path] : [];
  });
}

function concreteRunImports(root, path, source) {
  const violations = [];
  for (const capability of CONCRETE_RUN_CAPABILITIES) {
    const direct = `crate::spaces::${capability}`;
    const grouped = new RegExp(`\\bcrate::spaces::\\{[^}]*\\b${capability}\\b`, "s");
    if (source.includes(direct) || grouped.test(source)) {
      violations.push(`${relative(root, path)} imports concrete capability ${direct}`);
    }
  }
  return violations;
}

export function inspectRustBoundaries(root) {
  const violations = [];
  const localResources = join(root, "src-tauri/src/local_resources");
  for (const path of rustFiles(localResources)) {
    if (/\bcrate::ai\b|\bcrate::\{[^}]*\bai\b/s.test(readFileSync(path, "utf8"))) {
      violations.push(`${relative(root, path)} imports the AI capability`);
    }
  }
  const runs = join(root, "src-tauri/src/spaces/runs");
  for (const path of rustFiles(runs)) {
    const source = readFileSync(path, "utf8");
    if (/\bcrate::ai\b|\bcrate::\{[^}]*\bai\b/s.test(source)) {
      violations.push(`${relative(root, path)} imports concrete capability crate::ai`);
    }
    violations.push(...concreteRunImports(root, path, source));
  }
  const authorization = join(root, "src-tauri/src/spaces/authorization");
  for (const path of rustFiles(authorization)) {
    if (/\bcrate::spaces::actions\b/.test(readFileSync(path, "utf8"))) {
      violations.push(`${relative(root, path)} imports the Action catalog`);
    }
  }
  const workflowExecution = join(root, "src-tauri/src/spaces/workflows/execution");
  for (const path of rustFiles(workflowExecution)) {
    const source = readFileSync(path, "utf8");
    if (/\bcrate::spaces::speech\b/.test(source) || source.includes("execute_durable")) {
      violations.push(`${relative(root, path)} dispatches a non-captured Run capability`);
    }
  }
  return violations;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = process.cwd();
  const violations = inspectRustBoundaries(root);
  if (violations.length > 0) {
    console.error(violations.join("\n"));
    process.exitCode = 1;
  }
}
