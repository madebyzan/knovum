import { readFileSync } from "node:fs";
import { join } from "node:path";

const DOC = "docs/commands.md";
const PLUGIN_ADDRESS = /^plugin:[a-z0-9_-]+\|[a-z0-9_]+$/u;
const APP_ADDRESS = /^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/u;
const BACKTICKED = /`([^`\n]+)`/gu;

// `docs/` is part of the contract, so a command name it mentions must resolve: a
// dangling name invites a Renderer call that can only fail at runtime.
export function loadCommandDocs(root) {
  const addresses = new Set();
  for (const [, token] of readFileSync(join(root, DOC), "utf8").matchAll(BACKTICKED)) {
    if (PLUGIN_ADDRESS.test(token) || APP_ADDRESS.test(token)) addresses.add(token);
  }
  return addresses;
}

export function commandAddress(token) {
  return token.startsWith("plugin:") ? token.slice("plugin:".length) : token;
}
