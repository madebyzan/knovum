import { createHash } from "node:crypto";
import { cpSync, lstatSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
// PyInstaller links are preserved by the bundle, and never signed a second time through an alias.
function visit(at, prefix, allowLinks) {
  return readdirSync(at, { withFileTypes: true }).flatMap((entry) => {
    const name = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isSymbolicLink()) {
      if (allowLinks) return [];
      throw new Error("Native resource contains an undeclared link");
    }
    if (entry.isDirectory()) return visit(join(at, entry.name), name, allowLinks);
    if (!entry.isFile()) throw new Error("Bundle contains a special file");
    return [name];
  });
}
export function regularFiles(root, { allowLinks = false } = {}) {
  return visit(root, "", allowLinks).toSorted();
}
function hash(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
export function copyTree(source, destination) {
  rmSync(destination, { recursive: true, force: true });
  cpSync(source, destination, { recursive: true, verbatimSymlinks: true });
}
export function seal(root, manifestName, identity) {
  const files = regularFiles(root)
    .filter((path) => path !== manifestName)
    .map((path) => ({ path, size: lstatSync(join(root, path)).size, sha256: hash(join(root, path)) }));
  if (!files.length || files.length > 256) throw new Error("Native resource inventory exceeds its bound");
  const manifest = { schemaVersion: 1, ...identity, files };
  writeFileSync(join(root, manifestName), JSON.stringify(manifest));
  return manifest;
}
export function verify(root, manifestName, trusted) {
  const actual = JSON.parse(readFileSync(join(root, manifestName), "utf8"));
  if (JSON.stringify(actual) !== JSON.stringify(trusted))
    throw new Error("Packaged manifest differs from the controlled build");
  const names = new Set(trusted.files.map((file) => file.path));
  const inventory = regularFiles(root).filter((path) => path !== manifestName);
  if (inventory.length !== names.size || inventory.some((path) => !names.has(path)))
    throw new Error("Packaged inventory differs");
  for (const file of trusted.files) verifyFile(root, file);
}
function verifyFile(root, file) {
  if (
    !file.path ||
    file.path.includes("\\") ||
    file.path.split("/").some((part) => !part || part === ".." || part === ".")
  )
    throw new Error("Invalid inventory path");
  const path = join(root, file.path);
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.size !== file.size || hash(path) !== file.sha256)
    throw new Error("Packaged bytes differ");
}
