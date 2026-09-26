import { existsSync, readFileSync } from "node:fs";
import { dirname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const DECLARED_SOURCE =
  /(?:\.d\.ts$|\/contracts\/generated\/|\/locales\/|\/styles\/|\/__tests__\/|\/test-support\/)/u;

export function unreachableProductionModules(modules, workerEdges = new Map()) {
  const byPath = new Map(modules.map((module) => [module.source, module]));
  if (!byPath.has("src/main.ts")) throw new Error("Production graph is missing src/main.ts");
  const reachable = new Set();
  const pending = ["src/main.ts"];
  while (pending.length) {
    const path = pending.pop();
    if (reachable.has(path)) continue;
    reachable.add(path);
    for (const dependency of byPath.get(path)?.dependencies ?? []) {
      if (byPath.has(dependency.resolved)) pending.push(dependency.resolved);
    }
    pending.push(...(workerEdges.get(path) ?? []));
  }
  return [...byPath.keys()].filter(
    (path) => path.startsWith("src/") && !DECLARED_SOURCE.test(path) && !reachable.has(path),
  );
}

function collectWorkerEdges(modules) {
  const edges = new Map();
  for (const { source } of modules) {
    if (!source.startsWith("src/") || !existsSync(source)) continue;
    const text = readFileSync(source, "utf8");
    const urls = [...text.matchAll(/new URL\(["'](\.[^"']+)["'],\s*import\.meta\.url\)/gu)];
    const targets = urls
      .map((match) => normalize(join(dirname(source), match[1])).replaceAll("\\", "/"))
      .filter((target) => /\.(?:ts|js)$/u.test(target));
    edges.set(source, targets);
  }
  return edges;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { modules } = JSON.parse(readFileSync(0, "utf8"));
  const unused = unreachableProductionModules(modules, collectWorkerEdges(modules));
  if (unused.length) {
    console.error(`Production-unreachable source files (${unused.length}):\n${unused.join("\n")}`);
    process.exitCode = 1;
  } else console.log("Production source reachability passed (including Worker URL imports).");
}
