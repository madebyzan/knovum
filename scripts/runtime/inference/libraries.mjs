import { execFileSync } from "node:child_process";
import { copyFileSync, lstatSync, readdirSync, realpathSync, renameSync, rmSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";

export function materializeLibraryAliases(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name),
      metadata = lstatSync(path);
    if (metadata.isSymbolicLink()) {
      const target = realpathSync(path);
      if (!target.startsWith(`${realpathSync(directory)}/`))
        throw new Error("Inference install alias escapes its directory");
      const temporary = `${path}.materialized`;
      copyFileSync(target, temporary);
      rmSync(path);
      renameSync(temporary, path);
    } else if (metadata.isDirectory()) materializeLibraryAliases(path);
    else if (!metadata.isFile()) throw new Error("Inference install contains a non-file");
  }
}

export function relocateLibraries(directory, files) {
  const libraries = files.filter((name) => name.endsWith(".dylib") || name.endsWith(".so"));
  const aliases = new Map();
  for (const file of libraries) {
    const list = aliases.get(basename(file)) ?? [];
    list.push(file);
    aliases.set(basename(file), list);
  }
  for (const file of libraries) {
    if (file.startsWith("libonnxruntime.")) continue;
    relocateLibrary(directory, file, aliases);
  }
}

function relocateLibrary(directory, file, aliases) {
  const path = join(directory, file);
  const dependencies = execFileSync("otool", ["-L", path], { encoding: "utf8" })
    .trim()
    .split("\n")
    .slice(1)
    .map((line) => line.trim().split(" (")[0]);
  const installName = execFileSync("otool", ["-D", path], { encoding: "utf8" }).trim().split("\n")[1]?.trim();
  // MODULE libraries have no LC_ID_DYLIB; their first entry is a real dependency.
  if (installName) execFileSync("install_name_tool", ["-id", `@loader_path/${basename(file)}`, path]);
  for (const dependency of dependencies.filter((name) => name !== installName)) {
    if (dependency.startsWith("/usr/lib/") || dependency.startsWith("/System/Library/")) continue;
    const candidate = dependencyOwner(file, dependency, aliases);
    const destination = `@loader_path/${relative(dirname(file), candidate).replaceAll("\\", "/")}`;
    execFileSync("install_name_tool", ["-change", dependency, destination, path]);
  }
  const commands = execFileSync("otool", ["-l", path], { encoding: "utf8" });
  for (const match of commands.matchAll(/cmd LC_RPATH\s+cmdsize \d+\s+path (.*?) \(offset/gu)) {
    if (!match[1].startsWith("@loader_path"))
      execFileSync("install_name_tool", ["-delete_rpath", match[1], path]);
  }
  // Every mutation invalidates Mach-O signatures; seal the final bytes after signing.
  execFileSync("codesign", ["--force", "--sign", "-", path]);
}

function dependencyOwner(file, dependency, aliases) {
  const candidates = aliases.get(basename(dependency)) ?? [];
  const sameDirectory = candidates.filter((candidate) => dirname(candidate) === dirname(file));
  const owned = file.startsWith("compiled-libraries/whisper/")
    ? candidates.filter((candidate) => candidate.startsWith("compiled-libraries/whisper/"))
    : candidates.filter((candidate) => candidate.startsWith("bin/"));
  const candidate =
    sameDirectory.length === 1
      ? sameDirectory[0]
      : owned.length === 1
        ? owned[0]
        : candidates.length === 1
          ? candidates[0]
          : undefined;
  if (!candidate)
    throw new Error(`Inference dependency has no unambiguous packaged owner: ${file}: ${dependency}`);
  return candidate;
}
