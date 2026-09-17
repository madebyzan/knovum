import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { commandAddress, loadCommandDocs } from "./command-docs.mjs";
import { checkRustCommandExports } from "./rust-command-exports.mjs";
import { discoverPluginCommands, pluginKeyFor } from "./tauri-plugin-commands.mjs";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const rustRoot = join(root, "src-tauri", "src");
const frontendRoot = join(root, "src");
const permissionRoot = join(root, "src-tauri", "permissions");
const capabilityRoot = join(root, "src-tauri", "capabilities");

const rustFiles = walk(rustRoot, ".rs");
const frontendFiles = walk(frontendRoot, ".ts", ".vue");
const permissionFiles = walk(permissionRoot, ".toml");
const capabilityFiles = walk(capabilityRoot, ".json");

const pluginSurface = discoverPluginCommands(root, rustRoot, rustFiles);

// Tauri exposes two IPC namespaces: app commands resolve by bare name, plugin
// commands by `plugin:<key>|<command>`. Each has its own grant mechanism.
const appCommands = new Set();
const pluginCommands = new Set();
for (const file of rustFiles) {
  for (const name of commandNamesIn(readFileSync(file, "utf8"))) {
    const key = pluginKeyFor(pluginSurface, file, name);
    if (key) pluginCommands.add(`${key}|${name}`);
    else appCommands.add(name);
  }
}

const appBuilderSource = readFileSync(join(rustRoot, "app_builder.rs"), "utf8");
const handlerBlock = appBuilderSource.match(
  /\.invoke_handler\(tauri::generate_handler!\[([\s\S]*?)\]\)/u,
)?.[1];
const handlerPaths = (handlerBlock ?? "")
  .split(",")
  .map((entry) => entry.trim())
  .filter(Boolean)
  .map((entry) => (entry.startsWith("crate::") ? entry : `crate::${entry}`));
const handlers = new Set(handlerPaths.map((path) => path.split("::").at(-1)));

const permissions = new Set();
for (const file of permissionFiles) {
  const source = readFileSync(file, "utf8");
  for (const block of source.matchAll(/commands\.allow\s*=\s*\[([\s\S]*?)\]/gu)) {
    for (const quoted of block[1].matchAll(/"([A-Za-z0-9_]+)"/gu)) permissions.add(quoted[1]);
  }
}

const granted = new Set();
for (const file of capabilityFiles) {
  const capability = JSON.parse(readFileSync(file, "utf8"));
  for (const permission of capability.permissions ?? []) {
    granted.add(typeof permission === "string" ? permission : permission.identifier);
  }
}

// A command address is recognised anywhere in a Renderer file, not only inside
// `invoke(...)`, because a caller may hold it in a constant or a lookup table.
const owners = new Set(handlerPaths.map((path) => path.split("::")[1]));
const keys = new Set(pluginSurface.byKey.keys());
const pluginAddress = tokenPattern(`plugin:[a-z0-9_-]+\\|[a-z0-9_]+`);
const appAddress = owners.size > 0 ? tokenPattern(`(?:${[...owners].join("|")})_[a-z0-9_]+`) : null;
const legacyAddress = keys.size > 0 ? tokenPattern(`(?:${[...keys].join("|")})_[a-z0-9_]+`) : null;
const pluginInvokes = new Set();
const appInvokes = new Set();
const failures = pluginSurface.errors.map((error) => `Tauri plugin surface: ${error}`);

for (const file of frontendFiles) {
  const source = readFileSync(file, "utf8");
  const location = relative(root, file);
  for (const address of matched(source, pluginAddress)) {
    pluginInvokes.add(address.slice("plugin:".length));
  }
  for (const address of matched(source, appAddress)) appInvokes.add(address);
  for (const legacy of legacyAddress ? matched(source, legacyAddress) : []) {
    failures.push(
      `${location}: "${legacy}" addresses a plugin-owned command in the flat namespace; use plugin:<key>|<command>`,
    );
  }
}

const addresses = loadCommandDocs(root);

failures.push(...checkRustCommandExports(root, rustRoot, handlerPaths));
if (!handlerBlock) failures.push("src-tauri/src/app_builder.rs has no application invoke handler");
if (handlers.size !== handlerPaths.length) failures.push("application invoke handler lists a command twice");
if (!isSorted(handlerPaths)) failures.push("application invoke handler commands must be sorted");

checkSubset("Rust command missing invoke handler", appCommands, handlers, failures);
checkSubset("Invoke handler has no Rust command", handlers, appCommands, failures);
checkSubset("Rust command missing manual permission", appCommands, permissions, failures);
checkSubset("Frontend invoke has no Rust command", appInvokes, appCommands, failures);
checkSubset("Frontend invoke missing manual permission", appInvokes, permissions, failures);

checkSubset("Frontend invoke has no plugin command", pluginInvokes, pluginCommands, failures);
checkGrants("Plugin command is not granted to any capability", pluginCommands, failures);
checkGrants("Frontend plugin invoke is not granted to any capability", pluginInvokes, failures);

const declared = new Set([...pluginCommands, ...appCommands]);
for (const token of addresses) {
  if (!declared.has(commandAddress(token))) {
    failures.push(`docs/commands.md references "${token}", which is not a declared command`);
  }
}

console.log(
  `Tauri command coverage: app rust=${appCommands.size} handlers=${handlers.size} permissions=${permissions.size} frontend=${appInvokes.size}` +
    `; plugin commands=${pluginCommands.size} invokes=${pluginInvokes.size} plugins=${keys.size}`,
);
if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exitCode = 1;
}

function tokenPattern(shape) {
  return shape ? new RegExp(`\\b(?:${shape})\\b`, "gu") : null;
}

function matched(source, pattern) {
  return pattern ? new Set([...source.matchAll(pattern)].map((match) => match[0])) : new Set();
}

function isSorted(values) {
  return values.every((value, index) => index === 0 || values[index - 1] <= value);
}

function checkGrants(label, addressesToGrant, output) {
  const ungranted = [...addressesToGrant]
    .filter((address) => {
      const [key, command] = address.split("|");
      // tauri-build autogenerates the permission identifier as `allow-<command>`
      // with underscores slugified to dashes, while the runtime address keeps the
      // snake_case command. Mirror that shape so the grant lookup matches.
      const slug = command.replaceAll("_", "-");
      return !granted.has(`${key}:allow-${slug}`) && !granted.has(`${key}:default`);
    })
    .toSorted();
  if (ungranted.length > 0) output.push(`${label}: ${ungranted.join(", ")}`);
}

function checkSubset(label, required, available, output) {
  const missing = [...required].filter((value) => !available.has(value)).toSorted();
  if (missing.length > 0) output.push(`${label}: ${missing.join(", ")}`);
}

function commandNamesIn(source) {
  const pattern =
    /#\[tauri::command\][\s\S]{0,240}?\bpub(?:\(crate\))?\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)/gu;
  return [...source.matchAll(pattern)].map((match) => match[1]);
}

function walk(directory, ...extensions) {
  const files = [];
  for (const name of readdirSync(directory)) {
    const path = join(directory, name);
    const stat = statSync(path);
    if (stat.isDirectory()) files.push(...walk(path, ...extensions));
    else if (extensions.some((extension) => name.endsWith(extension))) files.push(path);
  }
  return files;
}
