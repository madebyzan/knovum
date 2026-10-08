import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const generatedContractOutputs = [
  "src/shared/contracts/generated",
  "src/features/spaces/contracts/generated",
  "src/features/dictation/contracts/generated",
  "python/schemas",
  "python/src/knovum_inference/contracts/generated",
  "python/src/knovum_agent_runtime/contracts",
  "python/src/knovum_model_resources/contracts.py",
];

export function generateContracts() {
  const commands = [
    [
      "cargo",
      ["run", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--bin", "export_space_contracts"],
    ],
    [
      "uv",
      [
        "run",
        "--project",
        "python",
        "--only-group",
        "contracts",
        "--frozen",
        "python",
        "scripts/generate_python_contracts.py",
      ],
    ],
  ];
  for (const [command, args] of commands) {
    const execution = spawnSync(command, args, {
      cwd: fileURLToPath(new URL("../", import.meta.url)),
      stdio: "inherit",
    });
    if (execution.error) throw execution.error;
    if (execution.status !== 0) process.exit(execution.status ?? 1);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) generateContracts();
