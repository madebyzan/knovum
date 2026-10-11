import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, posix } from "node:path";

import { compiledFiles } from "./compiled-signing.mjs";
import { listRuntimeFiles } from "./files.mjs";
import { dumpbinPath } from "./media-binary.mjs";

export function requireWindowsNativeClosure(directory, repository) {
  const inventory = listRuntimeFiles(directory);
  const allowed = new Set(
    JSON.parse(readFileSync(join(repository, "scripts/runtime/windows-system-libraries.json"), "utf8")),
  );
  const inspector = dumpbinPath();
  for (const file of compiledFiles(directory, "windows-x64")) {
    const output = execFileSync(inspector, ["/NOLOGO", "/DEPENDENTS", file], {
      encoding: "utf8",
      env: { ...process.env, VSLANG: "1033" },
      maxBuffer: 1024 * 1024,
    });
    const dependencies = [...output.matchAll(/^\s+([\w.+-]+\.(?:dll|drv))\s*$/gim)].map((match) => match[1]);
    validateWindowsImports(
      file.slice(directory.length + 1).replaceAll("\\", "/"),
      dependencies,
      inventory,
      allowed,
    );
  }
}

export function validateWindowsImports(file, dependencies, inventory, system) {
  if (!dependencies.length) throw new Error("Windows native dependency inspection returned no imports");
  const paths = new Set(inventory.map((path) => path.toLowerCase()));
  for (const value of dependencies) {
    const name = value.toLowerCase();
    if (!/^[\w.+-]+\.(?:dll|drv)$/u.test(name)) throw new Error("Invalid Windows import name");
    const local = posix.join(posix.dirname(file.toLowerCase()), name);
    if (
      paths.has(local) ||
      paths.has(`bin/${name}`) ||
      system.has(name) ||
      name.startsWith("api-ms-") ||
      name.startsWith("ext-ms-")
    )
      continue;
    throw new Error(`Unsealed Windows dependency: ${file} imports ${value}`);
  }
}
