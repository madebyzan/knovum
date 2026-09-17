import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { checkIpcNullability } from "./ipc-nullability.mjs";
import { checkRustSourceStructure } from "./rust-source-structure.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const rustRoot = join(root, "src-tauri", "src");
const FORBIDDEN_SPACE_PERMISSIONS = new Set(["fs:allow-write-file", "fs:allow-mkdir", "fs:allow-remove"]);

const failures = [];
const rustFiles = filesUnder(rustRoot, ".rs");
failures.push(...checkRustSourceStructure(root, rustFiles));
failures.push(...checkIpcNullability(root));

forbid(
  join(rustRoot, "bundled_runtime.rs"),
  /crate::(?:ai|spaces|desktop)\b/,
  "Shared bundled runtime must not depend on product capabilities",
);
forbid(join(rustRoot, "ai"), /crate::spaces\b/, "AI infrastructure must not depend on the Space feature");
forbid(
  join(rustRoot, "spaces", "queries"),
  /crate::spaces::commands\b|super::(?:super::)?commands\b/,
  "Space queries must not depend on mutation owners",
);
forbid(
  join(rustRoot, "spaces", "knowledge"),
  /crate::spaces::knowledge_derivations\b|super::(?:super::)?knowledge_derivations\b/,
  "Space knowledge policy must not depend on derivation use cases",
);

for (const failure of capabilityFailures(join(root, "src-tauri", "capabilities"))) failures.push(failure);

if (failures.length) {
  console.error(
    `Rust architecture gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}
console.log("Rust architecture gate: module direction, structure and capability scope are clean");

function filesUnder(directory, extension) {
  const output = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) output.push(...filesUnder(path, extension));
    else if (entry.isFile() && path.endsWith(extension)) output.push(path);
  }
  return output;
}

function rustFilesIn(path) {
  if (!existsSync(path)) return [];
  const stat = statSync(path);
  if (stat.isFile()) return path.endsWith(".rs") ? [path] : [];
  if (stat.isDirectory()) return filesUnder(path, ".rs");
  return [];
}

function forbid(path, pattern, reason) {
  for (const file of rustFilesIn(path)) {
    if (pattern.test(readFileSync(file, "utf8"))) failures.push(`${relative(root, file)}: ${reason}`);
  }
}

// The WebView ACL is the security boundary, and nothing else in the toolchain reads
// capabilities/*.json, so an over-broad grant here is invisible to cargo and to vue-tsc.
function capabilityFailures(directory) {
  const output = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
    const location = `src-tauri/capabilities/${entry.name}`;
    const capability = JSON.parse(readFileSync(join(directory, entry.name), "utf8"));
    for (const permission of permissionIdentifiers(capability)) {
      if (permission.startsWith("sql:")) {
        output.push(`${location}: WebView SQL permission ${permission} is forbidden`);
      }
      if (permission === "fs:default" && capability.identifier === "main-desktop") {
        output.push(`${location}: main window must not regain fs:default`);
      }
      if (capability.identifier === "spaces" && FORBIDDEN_SPACE_PERMISSIONS.has(permission)) {
        output.push(`${location}: Renderer persistent filesystem mutation ${permission} is forbidden`);
      }
    }
  }
  return output;
}

function permissionIdentifiers(capability) {
  return (capability.permissions ?? []).map((permission) =>
    typeof permission === "string" ? permission : permission.identifier,
  );
}
