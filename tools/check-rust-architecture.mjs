import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const rustRoot = join(root, "src-tauri", "src");
const FORBIDDEN_SPACE_PERMISSIONS = new Set(["fs:allow-write-file", "fs:allow-mkdir", "fs:allow-remove"]);

const failures = [];
const rustFiles = filesUnder(rustRoot, ".rs");

forbid(
  join(rustRoot, "documents"),
  /crate::spaces\b/,
  "Document engines must not depend on Space authorization or persistence",
);
forbid(
  join(rustRoot, "local_resources"),
  /crate::(?:ai|spaces|desktop)\b/,
  "Local resources must not depend on product capabilities",
);
forbid(join(rustRoot, "ai"), /crate::spaces\b/, "AI infrastructure must not depend on the Space feature");
forbid(join(rustRoot, "agent_runtime"), /crate::spaces\b/, "Agent Runtime must not depend on Space policy");
forbid(
  join(rustRoot, "external_agents"),
  /crate::spaces\b/,
  "External Agent protocol adapters must not depend on Space policy",
);
for (const file of rustFiles) {
  const source = readFileSync(file, "utf8");
  const relativePath = relative(rustRoot, file);
  if (!relativePath.startsWith("agent_runtime/rig/") && /\brig_agent::/.test(source)) {
    failures.push(`${relative(root, file)}: Rig vendor types must stay inside agent_runtime/rig`);
  }
}
forbid(
  join(rustRoot, "spaces", "workflows"),
  /crate::spaces::assistant\b|super::(?:super::)?assistant\b/,
  "Workflows must not depend on the Assistant implementation",
);
forbid(
  join(rustRoot, "spaces", "mcp"),
  /crate::spaces::db\b|super::(?:super::)?db\b/,
  "MCP adapters must use Space use cases instead of the database adapter",
);
forbid(
  join(rustRoot, "spaces", "queries"),
  /crate::spaces::commands\b|super::(?:super::)?commands\b/,
  "Space queries must not depend on mutation owners",
);
forbid(
  join(rustRoot, "spaces", "knowledge"),
  /crate::spaces::derivation\b|super::(?:super::)?derivation\b/,
  "Space knowledge policy must not depend on derivation use cases",
);
forbid(
  join(rustRoot, "spaces", "actions"),
  /crate::spaces::(?:nodes|structure|knowledge|assets|derivation)::/,
  "SpaceAction definitions cannot absorb concrete domain implementations",
);

for (const failure of capabilityFailures(join(root, "src-tauri", "capabilities"))) failures.push(failure);

if (failures.length) {
  console.error(
    `Rust architecture gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}
console.log("Rust architecture gate: module direction and capability policy checks passed");

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
