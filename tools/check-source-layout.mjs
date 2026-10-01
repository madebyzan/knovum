import { readdirSync } from "node:fs";
import { basename, dirname, extname, join } from "node:path";

const conflicts = [];
for (const root of ["src", "src-tauri/src", "tools"]) {
  const entries = readdirSync(root, { recursive: true, withFileTypes: true });
  const directories = new Set(
    entries.filter((entry) => entry.isDirectory()).map((entry) => join(entry.parentPath, entry.name)),
  );
  for (const entry of entries) {
    if (!entry.isFile()) continue;
    const file = join(entry.parentPath, entry.name);
    const directory = join(dirname(file), basename(file, extname(file)));
    if (directories.has(directory)) conflicts.push(`${file}: conflicts with directory ${directory}`);
  }
}

if (conflicts.length) {
  console.error(conflicts.join("\n"));
  process.exitCode = 1;
} else {
  console.log("Source layout: no sibling file/directory name conflicts.");
}
