import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const root = "src-tauri/src/spaces/knowledge";
const violations = [];

for (const entry of readdirSync(root, { recursive: true, withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith(".rs")) continue;
  const file = join(entry.parentPath, entry.name);
  const source = readFileSync(file, "utf8");
  for (const [index, line] of source.split("\n").entries()) {
    if (/\b(?:crate::)?spaces::(?:assets|queries)\b/.test(line)) {
      violations.push(`${file}:${index + 1}: knowledge must not import assets or queries`);
    }
  }
}

if (violations.length) {
  console.error(violations.join("\n"));
  process.exitCode = 1;
} else {
  console.log("Rust Spaces boundaries: knowledge has no assets or queries imports.");
}
