import { readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

// Runtime provider identities are generated from Rust; check their parser round-trip coverage.
const root = fileURLToPath(new URL("../", import.meta.url));
const catalogPath = join(root, "src/shared/contracts/generated/contract-values.ts");
const contractPath = join(root, "src-tauri/src/ai/contracts/provider.rs");

const offered = enumMembers(readFileSync(catalogPath, "utf8"), "providerKinds");
const recognized = acceptedProviderStrings(readFileSync(contractPath, "utf8"));
const failures = offered
  .filter((provider) => !recognized.has(provider))
  .map((provider) => `${relative(root, catalogPath)}: "${provider}" is not accepted by ProviderKind::parse`);

if (failures.length) {
  console.error(
    `AI provider catalog gate failed (${failures.length}):\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
  process.exit(1);
}

console.log(`AI provider catalog gate: all ${offered.length} offered providers resolve to a ProviderKind`);

function enumMembers(source, name) {
  const body = source.match(/export const providerKinds = \[([\s\S]*?)\] as const/u)?.[1];
  if (body === undefined) throw new Error(`Cannot read the ${name} catalog`);
  return [...body.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
}

function acceptedProviderStrings(source) {
  const body = source.match(/pub fn parse\(value: &str\) -> Option<Self> \{([\s\S]*?)\n {4}\}/u)?.[1];
  if (body === undefined) throw new Error("Cannot read ProviderKind::parse");
  return new Set([...body.matchAll(/"([^"]+)"/gu)].map((match) => match[1]));
}
