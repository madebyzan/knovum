import { spawnSync } from "node:child_process";

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
  "--extra",
  "model-resources",
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
  const result = spawnSync("uv", ["run", ...environment, ...command], { stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
