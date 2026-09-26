import { existsSync, statSync } from "node:fs";
import { basename, relative, sep } from "node:path";

export function checkFrontendSourceNaming(root, sourceRoot, files) {
  const failures = [];

  for (const file of files) {
    const segments = relative(sourceRoot, file).split(sep);
    checkDirectories(root, file, segments.slice(0, -1), failures);

    const name = basename(file);
    const stem = file.replace(/\.(?:ts|vue)$/u, "");
    if (existsSync(stem) && statSync(stem).isDirectory())
      failures.push(`${relative(root, file)}: same-name source file and directory cannot coexist`);
    if (name === "index.ts") {
      failures.push(`${relative(root, file)}: index.ts is forbidden; name the file after its responsibility`);
      continue;
    }

    const suffix = suffixFailure(root, file, name, segments);
    if (suffix) failures.push(suffix);
  }

  return failures;
}

function checkDirectories(root, file, directories, failures) {
  for (const directory of directories) {
    if (directory !== "__tests__" && !isKebabCase(directory)) {
      failures.push(`${relative(root, file)}: frontend source directories must use kebab-case`);
      return;
    }
  }
}

function suffixFailure(root, file, name, segments) {
  if (name.endsWith(".test.ts")) {
    return segments.includes("__tests__")
      ? undefined
      : `${relative(root, file)}: unit tests must live under the owning __tests__ directory`;
  }
  if (segments.includes("__tests__")) {
    return `${relative(root, file)}: files under __tests__ must use the exact .test.ts suffix`;
  }
  if (name.endsWith(".ts") && !isTypeScriptName(name)) {
    return `${relative(root, file)}: TypeScript source files must use kebab-case`;
  }
  if (name.endsWith(".vue") && !isPascalCase(name.slice(0, -4))) {
    return `${relative(root, file)}: Vue component files must use PascalCase`;
  }
  return undefined;
}

function isTypeScriptName(name) {
  return isKebabCase(name.endsWith(".d.ts") ? name.slice(0, -5) : name.slice(0, -3));
}

function isKebabCase(value) {
  return /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/u.test(value);
}

function isPascalCase(value) {
  return /^[A-Z][A-Za-z0-9]*$/u.test(value);
}
