import { readFileSync, existsSync, statSync } from "node:fs";
import { basename, dirname, join, relative, sep } from "node:path";

import { maskRust } from "./rust-source-mask.mjs";

const MAX_FILE_LINES = 180;
const MAX_FUNCTION_PARAMS = 7;

export function checkRustSourceStructure(root, files) {
  const failures = [];
  const paths = new Set(files);
  const testDirectories = new Set();
  for (const file of files) {
    const source = readFileSync(file, "utf8");
    checkModuleLayout({ root, file, source, paths, testDirectories, failures });
    const commentsMasked = maskRust(source, false);
    const structureMasked = maskRust(source, true);
    const fileLines = effectiveLines(commentsMasked);
    if (fileLines > MAX_FILE_LINES) {
      failures.push(`${relative(root, file)}: ${fileLines} effective lines exceeds ${MAX_FILE_LINES}`);
    }
    for (const fn of functionsIn(source, structureMasked)) {
      if (fn.params > MAX_FUNCTION_PARAMS) {
        failures.push(
          `${relative(root, file)}:${fn.line}: ${fn.name} has ${fn.params} parameters; maximum is ${MAX_FUNCTION_PARAMS}`,
        );
      }
    }
  }
  for (const directory of testDirectories) {
    if (!paths.has(join(directory, "mod.rs"))) {
      failures.push(`${relative(root, directory)}: tests directory requires mod.rs`);
    }
  }
  return failures;
}

function checkModuleLayout(context) {
  const { root, file, paths, failures } = context;
  const name = basename(file);
  const directory = dirname(file);
  const segments = file.split(sep);
  const layout = { ...context, name, directory, segments };
  checkModuleNaming(layout);
  checkTestModuleLayout(layout);
  if (name !== "mod.rs") {
    checkFileDirectoryCollision(layout);
    return;
  }
  checkPrefixedSiblings(root, directory, paths, failures);
}

function checkModuleNaming({ root, file, name, directory, failures }) {
  const moduleSegments = relative(join(root, "src-tauri", "src"), directory)
    .split(sep)
    .filter(Boolean);
  for (const segment of moduleSegments) {
    if (!isSnakeCase(segment)) {
      failures.push(`${relative(root, directory)}: Rust module directories must use snake_case`);
      break;
    }
  }
  if (name !== "mod.rs" && !isSnakeCase(name.slice(0, -3))) {
    failures.push(`${relative(root, file)}: Rust source files must use snake_case`);
  }
}

function checkTestModuleLayout({ root, file, name, source, segments, testDirectories, failures }) {
  const testsIndex = segments.lastIndexOf("tests");
  if (testsIndex >= 0) testDirectories.add(segments.slice(0, testsIndex + 1).join(sep));
  if (name === "tests.rs" || name.endsWith("_tests.rs")) {
    failures.push(`${relative(root, file)}: tests must live under the owning module's tests/ directory`);
  }
  const masked = maskRust(source, true);
  if (/^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+tests\s*\{/mu.test(masked)) {
    failures.push(`${relative(root, file)}: inline test modules are forbidden`);
  }
  if (/#\s*\[\s*path\s*=/u.test(masked)) {
    failures.push(`${relative(root, file)}: #[path] module indirection is forbidden`);
  }
  const declarations = [...masked.matchAll(/^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+tests\s*;/gmu)];
  if (declarations.length > 1) {
    const lines = declarations.map((match) => lineNumber(source, match.index)).join(", ");
    failures.push(`${relative(root, file)}:${lines}: mod tests may be declared only once per module`);
  }
}

function checkFileDirectoryCollision({ root, file, name, directory, failures }) {
  const stem = name.slice(0, -3);
  if (existsSync(join(directory, stem)) && statSync(join(directory, stem)).isDirectory()) {
    failures.push(`${relative(root, file)}: same-name source file and directory cannot coexist`);
  }
}

function checkPrefixedSiblings(root, moduleDirectory, paths, failures) {
  const parent = dirname(moduleDirectory);
  const moduleName = basename(moduleDirectory);
  for (const candidate of paths) {
    if (dirname(candidate) === parent && basename(candidate).startsWith(`${moduleName}_`)) {
      failures.push(
        `${relative(root, candidate)}: prefixed sibling belongs inside ${relative(root, moduleDirectory)}`,
      );
    }
  }
}

function isSnakeCase(value) {
  return /^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$/u.test(value);
}

function functionsIn(source, masked) {
  const output = [];
  const pattern =
    /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:async|const|unsafe)\s+)*(?:extern\s+"[^"]+"\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/gmu;
  for (const match of masked.matchAll(pattern)) {
    const open = masked.indexOf("(", match.index);
    const close = closingDelimiter(masked, open, "(", ")");
    if (close < 0) continue;
    const bodyOpen = findBodyOpen(masked, close + 1);
    if (bodyOpen < 0) continue;
    if (closingDelimiter(masked, bodyOpen, "{", "}") < 0) continue;
    output.push({
      name: match[1],
      line: lineNumber(source, match.index),
      params: topLevelParameterCount(masked.slice(open + 1, close)),
    });
  }
  return output;
}

function findBodyOpen(source, from) {
  for (let index = from; index < source.length; index += 1) {
    if (source[index] === ";") return -1;
    if (source[index] === "{") return index;
  }
  return -1;
}

function closingDelimiter(source, open, left, right) {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === left) depth += 1;
    else if (source[index] === right) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function topLevelParameterCount(source) {
  let depth = 0;
  let count = 0;
  let hasToken = false;
  for (const char of source) {
    if ("([{<".includes(char)) depth += 1;
    else if (")]}>".includes(char)) depth = Math.max(0, depth - 1);
    else if (char === "," && depth === 0) {
      if (hasToken) count += 1;
      hasToken = false;
      continue;
    }
    if (!/\s/u.test(char)) hasToken = true;
  }
  return count + (hasToken ? 1 : 0);
}

function effectiveLines(source) {
  return source.split(/\r?\n/u).filter((line) => line.trim().length > 0).length;
}

function lineNumber(source, index) {
  return source.slice(0, index).split("\n").length;
}
