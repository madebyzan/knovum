import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const layout = spawnSync(
  process.execPath,
  [fileURLToPath(new URL("./source-layout.mjs", import.meta.url)), "python"],
  { stdio: "inherit" },
);
if (layout.error) throw layout.error;
if (layout.status !== 0) process.exit(layout.status ?? 1);

const configuration = ["--config", "python/pyproject.toml"];
const foundation = [
  "--project",
  "python",
  "--no-default-groups",
  "--group",
  "quality",
  "--group",
  "acceptance",
  "--group",
  "publication",
  "--frozen",
];
const agentRuntime = ["--project", "python/runtimes/agent", "--group", "quality", "--frozen"];
const checks = [
  [foundation, "ruff", "check", ...configuration, "python/src", "python/tests", "scripts"],
  [foundation, "ruff", "format", ...configuration, "--check", "python/src", "python/tests", "scripts"],
  [
    foundation,
    "mypy",
    "--config-file",
    "python/pyproject.toml",
    "--exclude",
    "knovum_agent_runtime",
    "python/src",
    "python/tests/acceptance",
    "scripts/runtime",
    "scripts/generate_python_contracts.py",
  ],
  [agentRuntime, "mypy", "--config-file", "python/pyproject.toml", "python/src/knovum_agent_runtime"],
  [
    foundation,
    "pytest",
    "-c",
    "python/pyproject.toml",
    "python/tests",
    "scripts",
    "--ignore=python/tests/agent_runtime",
  ],
  [agentRuntime, "pytest", "-c", "python/pyproject.toml", "python/tests/agent_runtime"],
];
for (const [environment, ...command] of checks) {
  const checkExecution = spawnSync("uv", ["run", ...environment, ...command], { stdio: "inherit" });
  if (checkExecution.error) throw checkExecution.error;
  if (checkExecution.status !== 0) process.exit(checkExecution.status ?? 1);
}
