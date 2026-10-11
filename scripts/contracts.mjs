import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repository = fileURLToPath(new URL("../", import.meta.url));

export const generatedContractOutputs = [
  "src/shared/contracts/generated",
  "src/features/spaces/contracts/generated",
  "src/features/settings/contracts/generated",
  "src/features/dictation/contracts/generated",
  "python/schemas/agent_runtime.schema.json",
  "src-tauri/crates/inference-contracts/inference.schema.json",
  "python/src/knovum_agent_runtime/contracts",
];

function snapshot() {
  const files = generatedContractOutputs.flatMap((relative) => {
    const path = resolve(repository, relative);
    if (!existsSync(path)) return [];
    if (statSync(path).isFile()) return [path];
    return readdirSync(path, { recursive: true })
      .filter((name) => /\.(ts|json|py)$/.test(name))
      .map((name) => join(path, name));
  });
  return JSON.stringify(
    files.toSorted().map((path) => [path, createHash("sha256").update(readFileSync(path)).digest("hex")]),
  );
}

export function generateContracts() {
  const commands = [
    [
      "cargo",
      ["run", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--package", "knovum-contract-exporter"],
    ],
    [
      "uv",
      [
        "run",
        "--project",
        "python",
        "--group",
        "contracts",
        "--locked",
        "python",
        "python/contracts/generate_python_contracts.py",
      ],
    ],
  ];
  for (const [command, args] of commands) {
    const execution = spawnSync(command, args, {
      cwd: repository,
      stdio: "inherit",
    });
    if (execution.error) throw execution.error;
    if (execution.status !== 0) process.exit(execution.status ?? 1);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const before = snapshot();
  generateContracts();
  if (process.argv.includes("--check") && before !== snapshot())
    throw new Error(
      "Generated contracts differ from the current project sources; review the regenerated files",
    );
}
