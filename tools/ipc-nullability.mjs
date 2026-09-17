import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

export function checkIpcNullability(root) {
  const rustRoot = join(root, "src-tauri", "src");
  const frontendRoot = join(root, "src");
  const rustContracts = serializedOptionFields(filesUnder(rustRoot, ".rs"));
  const interfaces = typescriptInterfaces(filesUnder(frontendRoot, ".ts", ".vue"));
  const failures = [];
  for (const contract of rustContracts) {
    for (const definition of interfaces.get(contract.name) ?? []) {
      const field = definition.fields.get(contract.field);
      if (!field) continue;
      if (field.optional || !/(?:^|\W)null(?:\W|$)/u.test(field.type)) {
        failures.push(
          `${relative(root, definition.file)}:${field.line}: ${contract.name}.${contract.field} must be required and include null because Rust serializes Option<T> as null`,
        );
      }
    }
  }
  return failures;
}

function serializedOptionFields(files) {
  const output = [];
  for (const file of files) {
    const source = readFileSync(file, "utf8");
    const pattern =
      /#\[derive\(([^\]]*\bSerialize\b[^\]]*)\)\][\s\S]{0,260}?(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z0-9_]+)\s*\{/gu;
    for (const match of source.matchAll(pattern)) {
      const open = source.indexOf("{", match.index);
      const close = closingBrace(source, open);
      if (close < 0) continue;
      const body = source.slice(open + 1, close);
      const camelCase = source.slice(match.index, open).includes('rename_all = "camelCase"');
      const fieldPattern =
        /(?:#\[serde\(([^\]]*)\)\]\s*)?(?:pub(?:\([^)]*\))?\s+)?([A-Za-z0-9_]+)\s*:\s*Option\s*</gu;
      for (const field of body.matchAll(fieldPattern)) {
        if ((field[1] ?? "").includes("skip_serializing_if")) continue;
        output.push({
          name: match[2],
          field: camelCase ? snakeToCamel(field[2]) : field[2],
        });
      }
    }
  }
  return output;
}

function typescriptInterfaces(files) {
  const output = new Map();
  for (const file of files) {
    const source = readFileSync(file, "utf8");
    const pattern = /(?:export\s+)?interface\s+([A-Za-z0-9_]+)(?:\s+extends[^{]+)?\s*\{/gu;
    for (const match of source.matchAll(pattern)) {
      const open = source.indexOf("{", match.index);
      const close = closingBrace(source, open);
      if (close < 0) continue;
      const body = source.slice(open + 1, close);
      const fields = new Map();
      const fieldPattern = /^\s*([A-Za-z0-9_]+)(\?)?\s*:\s*([^;\n]+)[;]?/gmu;
      for (const field of body.matchAll(fieldPattern)) {
        const offset = open + 1 + field.index;
        fields.set(field[1], {
          optional: Boolean(field[2]),
          type: field[3].trim(),
          line: source.slice(0, offset).split("\n").length,
        });
      }
      const definitions = output.get(match[1]) ?? [];
      definitions.push({ file, fields });
      output.set(match[1], definitions);
    }
  }
  return output;
}

function closingBrace(source, open) {
  let depth = 0;
  const stringState = { delimiter: null, escaped: false };
  for (let index = open; index < source.length; index += 1) {
    const char = source[index];
    if (consumeStringCharacter(stringState, char)) continue;
    if (char === '"' || char === "'" || char === "`") {
      stringState.delimiter = char;
      continue;
    }
    if (char === "{") depth += 1;
    else if (char === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function consumeStringCharacter(state, char) {
  if (!state.delimiter) return false;
  if (state.escaped) state.escaped = false;
  else if (char === "\\") state.escaped = true;
  else if (char === state.delimiter) state.delimiter = null;
  return true;
}

function snakeToCamel(value) {
  return value.replace(/_([a-z])/gu, (_, char) => char.toUpperCase());
}

function filesUnder(directory, ...extensions) {
  const output = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) output.push(...filesUnder(path, ...extensions));
    else if (extensions.some((extension) => path.endsWith(extension))) output.push(path);
  }
  return output;
}
