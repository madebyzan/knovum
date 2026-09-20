import { basename, dirname, relative, sep } from "node:path";

export function checkFrontendSourceNaming(root, sourceRoot, files) {
  const failures = [];
  const sourceOwners = collectSourceOwners(files);

  for (const file of files) {
    const segments = relative(sourceRoot, file).split(sep);
    checkDirectories(root, file, segments.slice(0, -1), failures);

    const name = basename(file);
    if (name === "index.ts") {
      failures.push(`${relative(root, file)}: index.ts is forbidden; name the file after its responsibility`);
      continue;
    }

    const suffix = suffixFailure(root, file, name, segments);
    if (suffix) failures.push(suffix);
    if (name.endsWith(".test.ts")) {
      checkTestOwner(root, file, name, sourceOwners, failures);
    }
  }

  return failures;
}

function collectSourceOwners(files) {
  const owners = new Map();
  for (const file of files) {
    const name = basename(file);
    if (file.includes(`${sep}__tests__${sep}`) || name.endsWith(".test.ts")) continue;
    const stem = sourceStem(name);
    if (!stem) continue;
    const candidates = owners.get(stem) ?? [];
    candidates.push(dirname(file));
    owners.set(stem, candidates);
  }
  return owners;
}

function sourceStem(name) {
  if (name.endsWith(".vue")) return name.slice(0, -4);
  if (name.endsWith(".d.ts")) return undefined;
  if (name.endsWith(".ts")) return name.slice(0, -3);
  return undefined;
}

function checkTestOwner(root, file, name, sourceOwners, failures) {
  const stem = name.slice(0, -".test.ts".length);
  const owners = sourceOwners.get(stem) ?? [];
  if (owners.length !== 1) return;
  const actualOwner = dirname(dirname(file));
  if (actualOwner !== owners[0]) {
    failures.push(
      `${relative(root, file)}: unit test for ${stem} must live under ${relative(root, owners[0])}/__tests__`,
    );
  }
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
      : `${relative(root, file)}: unit tests must live under an __tests__ directory`;
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
