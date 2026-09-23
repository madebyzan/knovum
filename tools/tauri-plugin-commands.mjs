import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

const DECLARED_PLUGIN =
  /\.plugin\(\s*"([a-z0-9_-]+)"\s*,\s*tauri_build::InlinedPlugin::new\(\)\s*\.commands\(&\[([\s\S]*?)\]\s*\)/gu;
const PLUGIN_BUILDER = /tauri::plugin::Builder::new\(\s*"([a-z0-9_-]+)"\s*\)([\s\S]*?)\.build\(\)/gu;
const HANDLER_LIST = /generate_handler!\[([\s\S]*?)\]/gu;
const HANDLER_FACTORY = /\b([A-Za-z0-9_:]+)::handler\(\)/gu;
const COMMAND_DECL =
  /#\[tauri::command\][\s\S]{0,240}?\bpub(?:\(crate\))?\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)/gu;

export function pluginKeyFor(surface, file, name) {
  return surface.byFile.get(file)?.get(name);
}

export function discoverPluginCommands(root, rustRoot, rustFiles) {
  const state = { byKey: new Map(), byFile: new Map(), errors: [] };
  const declared = readDeclaredPlugins(join(root, "src-tauri", "build.rs"), state.errors);
  const sources = new Map(rustFiles.map((file) => [file, readFileSync(file, "utf8")]));
  const context = { rustRoot, sources, state };

  for (const [file, source] of sources) {
    for (const [, key, chain] of source.matchAll(PLUGIN_BUILDER)) {
      registerPlugin(context, file, key, chain);
    }
  }
  reconcile(declared, state);
  return state;
}

function readDeclaredPlugins(buildScript, errors) {
  if (!existsSync(buildScript)) return new Map();
  const declared = new Map();
  for (const [, key, body] of readFileSync(buildScript, "utf8").matchAll(DECLARED_PLUGIN)) {
    const names = [...body.matchAll(/"([A-Za-z0-9_]+)"/gu)].map((name) => name[1]);
    if (names.length === 0) errors.push(`src-tauri/build.rs: inlined plugin "${key}" lists no commands`);
    declared.set(key, names.toSorted());
  }
  return declared;
}

function registerPlugin(context, file, key, chain) {
  const { rustRoot, sources, state } = context;
  const entries = collectHandlers(context, file, chain, new Set());
  if (entries.length === 0) {
    state.errors.push(`plugin "${key}": invoke handler registers no generate_handler! commands`);
    return;
  }
  for (const entry of entries) {
    const resolved = resolveHandler(rustRoot, file, entry);
    if (resolved.error) {
      state.errors.push(resolved.error);
      continue;
    }
    const implemented = commandNamesIn(sources.get(resolved.file) ?? "");
    if (!implemented.has(resolved.name)) {
      state.errors.push(`plugin "${key}": handler ${entry} is not a #[tauri::command] in its module`);
      continue;
    }
    if (!state.byKey.has(key)) state.byKey.set(key, new Set());
    state.byKey.get(key).add(resolved.name);
    if (!state.byFile.has(resolved.file)) state.byFile.set(resolved.file, new Map());
    state.byFile.get(resolved.file).set(resolved.name, key);
  }
}

function collectHandlers(context, file, source, visited) {
  if (visited.has(file)) return [];
  visited.add(file);
  const entries = [...source.matchAll(HANDLER_LIST)].flatMap((match) => splitHandlers(match[1]));
  for (const match of source.matchAll(HANDLER_FACTORY)) {
    const resolved = resolveHandler(context.rustRoot, file, `${match[1]}::handler`);
    if (resolved.error) {
      context.state.errors.push(resolved.error);
      continue;
    }
    entries.push(
      ...collectHandlers(context, resolved.file, context.sources.get(resolved.file) ?? "", visited),
    );
  }
  return entries;
}

function splitHandlers(body) {
  return body
    .replace(/\/\/[^\n]*/gu, "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

function resolveHandler(rustRoot, file, entry) {
  const segments = entry.split("::").filter(Boolean);
  const name = segments.at(-1);
  if (segments.length === 1) return { file, name };

  let baseDir;
  if (segments[0] === "crate") {
    baseDir = rustRoot;
    segments.shift();
  } else if (segments[0] === "self" || segments[0] === "super") {
    baseDir = dirname(file);
    while (segments[0] === "super") {
      baseDir = dirname(baseDir);
      segments.shift();
    }
    if (segments[0] === "self") segments.shift();
  } else {
    baseDir = dirname(file);
  }

  segments.pop();
  if (segments.length === 0) return { file, name };
  const modulePath = join(baseDir, ...segments);
  for (const candidate of [`${modulePath}.rs`, join(modulePath, "mod.rs")]) {
    if (existsSync(candidate)) return { file: candidate, name };
  }
  return { error: `plugin handler ${entry}: module path does not resolve to a Rust file` };
}

function commandNamesIn(source) {
  return new Set([...source.matchAll(COMMAND_DECL)].map((match) => match[1]));
}

function reconcile(declared, state) {
  const { byKey, errors } = state;
  for (const key of byKey.keys()) {
    if (!declared.has(key)) {
      errors.push(
        `plugin "${key}": registers commands but src-tauri/build.rs declares no InlinedPlugin, so its ACL is never generated`,
      );
    }
  }
  for (const [key, names] of declared) {
    const registered = byKey.get(key);
    if (!registered) {
      errors.push(`plugin "${key}": src-tauri/build.rs declares an InlinedPlugin no builder registers`);
      continue;
    }
    const actual = [...registered].toSorted();
    if (JSON.stringify(actual) !== JSON.stringify(names)) {
      errors.push(
        `plugin "${key}": build.rs commands [${names.join(", ")}] differ from generate_handler! commands [${actual.join(", ")}]`,
      );
    }
  }
}
