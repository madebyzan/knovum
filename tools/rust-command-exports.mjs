import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

export function checkRustCommandExports(root, rustRoot, handlerPaths) {
  const modules = loadModules(rustRoot);
  resolveReexports(modules);
  return handlerPaths.flatMap((handler) => {
    const segments = handler.split("::");
    if (segments.shift() !== "crate") return [`application invoke handler: invalid path ${handler}`];
    const command = segments.pop();
    const module = modules.get(segments.join("::"));
    return module?.exports.has(command)
      ? []
      : [`application invoke handler: ${handler} does not resolve to an exported #[tauri::command]`];
  });
}

function loadModules(rustRoot) {
  const modules = new Map();
  for (const file of rustFiles(rustRoot)) {
    const parts = moduleParts(rustRoot, file);
    const source = readFileSync(file, "utf8");
    const direct = new Set(
      [
        ...source.matchAll(
          /#\[tauri::command\][\s\S]{0,240}?\bpub(?:\(crate\))?\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)/gu,
        ),
      ].map((match) => match[1]),
    );
    modules.set(parts.join("::"), {
      parts,
      exports: new Set(direct),
      uses: [...source.matchAll(/\bpub\s+use\s+([\s\S]*?);/gu)].map((match) =>
        match[1].replace(/\s+/gu, " ").trim(),
      ),
    });
  }
  return modules;
}

function resolveReexports(modules) {
  for (let pass = 0; pass < modules.size; pass += 1) {
    if (!propagateReexports(modules)) return;
  }
}

function propagateReexports(modules) {
  let changed = false;
  for (const module of modules.values()) {
    for (const clause of module.uses) {
      for (const item of parseUse(module.parts, clause)) {
        changed = applyReexport(modules, module, item) || changed;
      }
    }
  }
  return changed;
}

function applyReexport(modules, module, item) {
  const target = modules.get(item.target);
  if (!target) return false;
  if (!item.wildcard) {
    return target.exports.has(item.name) && add(module.exports, item.alias);
  }
  let changed = false;
  for (const name of target.exports) changed = add(module.exports, name) || changed;
  return changed;
}

function parseUse(parts, clause) {
  const brace = clause.indexOf("{");
  if (brace >= 0) return parseBraceUse(parts, clause, brace);
  if (clause.endsWith("::*")) {
    return [{ target: resolveModule(parts, clause.slice(0, -3)), wildcard: true }];
  }
  const separator = clause.lastIndexOf("::");
  if (separator < 0) return [];
  const target = resolveModule(parts, clause.slice(0, separator));
  const [name, alias = name] = clause.slice(separator + 2).split(/\s+as\s+/u);
  return [{ target, name: name.trim(), alias: alias.trim(), wildcard: false }];
}

function parseBraceUse(parts, clause, brace) {
  const prefix = clause.slice(0, brace).replace(/::$/u, "").trim();
  const target = resolveModule(parts, prefix);
  const body = clause.slice(brace + 1, clause.lastIndexOf("}"));
  return body
    .split(",")
    .map((raw) => raw.trim())
    .filter(Boolean)
    .map((item) => {
      if (item === "*") return { target, wildcard: true };
      const [name, alias = name] = item.split(/\s+as\s+/u);
      return { target, name: name.trim(), alias: alias.trim(), wildcard: false };
    });
}

function resolveModule(parts, raw) {
  const segments = raw.split("::").filter(Boolean);
  const base = [...parts];
  if (segments[0] === "crate") {
    base.length = 0;
    segments.shift();
  } else if (segments[0] === "self") {
    segments.shift();
  } else {
    while (segments[0] === "super") {
      base.pop();
      segments.shift();
    }
  }
  return [...base, ...segments].join("::");
}

function moduleParts(rustRoot, file) {
  let name = relative(rustRoot, file).replaceAll("\\", "/").replace(/\.rs$/u, "");
  if (name.endsWith("/mod")) name = name.slice(0, -4);
  return name === "lib" ? [] : name.split("/");
}

function rustFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return rustFiles(path);
    return entry.isFile() && entry.name.endsWith(".rs") ? [path] : [];
  });
}

function add(set, value) {
  if (set.has(value)) return false;
  set.add(value);
  return true;
}
