import { spawnSync } from "node:child_process";

import { generateContracts, generatedContractOutputs } from "../contracts.mjs";

generateContracts();
const diff = spawnSync("git", ["diff", "--exit-code", "--", ...generatedContractOutputs], {
  stdio: "inherit",
});
if (diff.error) throw diff.error;
if (diff.status !== 0) process.exit(diff.status ?? 1);
const untracked = spawnSync(
  "git",
  ["ls-files", "--others", "--exclude-standard", "--", ...generatedContractOutputs],
  { encoding: "utf8" },
);
if (untracked.error) throw untracked.error;
if (untracked.status !== 0) process.exit(untracked.status ?? 1);
if (untracked.stdout.trim()) throw new Error(`Uncommitted generated contracts:\n${untracked.stdout}`);
