import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const FORBIDDEN_SPACE_PERMISSIONS = new Set(["fs:allow-write-file", "fs:allow-mkdir", "fs:allow-remove"]);
const failures = capabilityFailures(join(root, "src-tauri", "capabilities"));

if (failures.length) {
  console.error(`Capability policy check failed:\n${failures.join("\n")}`);
  process.exitCode = 1;
} else {
  console.log("Capability policy check passed.");
}

// Check explicit project permission policy; Tauri validates capability schemas.
// This check does not expand permission sets or prove effective window authorization.
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
