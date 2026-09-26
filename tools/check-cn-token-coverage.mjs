import { readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

// tailwind-merge classifies every non-builtin `text-*` as a colour, so cn.ts has to declare the
// tokens.css font sizes, radii and shadows or they lose conflicts silently.
const root = fileURLToPath(new URL("../", import.meta.url));
const tokensPath = join(root, "src/styles/tokens.css");
const cnPath = join(root, "src/shared/ui/cn.ts");

const namespaces = [
  { token: "text", declared: "TEXT_SIZES" },
  { token: "radius", declared: "RADII" },
  { token: "shadow", declared: "SHADOWS" },
];

const tokensSource = readFileSync(tokensPath, "utf8");
const cnSource = readFileSync(cnPath, "utf8");
const failures = namespaces.flatMap(({ token, declared }) => {
  const defined = tokenNames(tokensSource, token);
  const covered = new Set(declaredNames(cnSource, declared));
  const missing = defined
    .filter((name) => !covered.has(name))
    .map((name) => `${relative(root, cnPath)}: --${token}-${name} is not in ${declared}`);
  const stale = [...covered]
    .filter((name) => !defined.includes(name))
    .map((name) => `${relative(root, cnPath)}: ${declared} declares unknown --${token}-${name}`);
  return missing.concat(stale);
});

if (failures.length) {
  console.error(
    `cn token coverage gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}

console.log("cn token coverage gate: every text, radius and shadow token is a tailwind-merge class group");

function tokenNames(source, namespace) {
  const pattern = new RegExp(`^\\s*--${namespace}-([a-z0-9]+(?:-[a-z0-9]+)*):`, "gmu");
  const names = [...source.matchAll(pattern)].map((match) => match[1]);
  if (!names.length) throw new Error(`Cannot read the --${namespace}-* tokens`);
  return names;
}

function declaredNames(source, name) {
  const body = source.match(new RegExp(`const ${name} = \\[([\\s\\S]*?)\\]`, "u"))?.[1];
  if (body === undefined) throw new Error(`Cannot read the ${name} class group`);
  return [...body.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
}
