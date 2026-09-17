import { readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

// The settings UI lets a user pick any provider in `aiProviderSchema`, and that string crosses IPC
// verbatim. Rust resolves it through `ProviderKind::parse`, which returns `None` for anything it does
// not know, so a provider added to the frontend alone becomes a connection that can never run.
// `ProviderKind` also covers internal targets such as `local_candle` that are deliberately not
// user-selectable, so only the frontend-to-Rust direction is a contract.
const root = fileURLToPath(new URL("../", import.meta.url));
const catalogPath = join(root, "src/foundation/ai/ai-models.ts");
const contractPath = join(root, "src-tauri/src/ai/contracts/provider.rs");

const offered = enumMembers(readFileSync(catalogPath, "utf8"), "aiProviderSchema");
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
  const body = source.match(new RegExp(`const ${name} = z\\.enum\\(\\[([\\s\\S]*?)\\]\\)`, "u"))?.[1];
  if (body === undefined) throw new Error(`Cannot read the ${name} catalog`);
  return [...body.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
}

function acceptedProviderStrings(source) {
  const body = source.match(/pub fn parse\(value: &str\) -> Option<Self> \{([\s\S]*?)\n {4}\}/u)?.[1];
  if (body === undefined) throw new Error("Cannot read ProviderKind::parse");
  return new Set([...body.matchAll(/"([^"]+)"/gu)].map((match) => match[1]));
}
