import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { runtimeProfiles } from "../runtime/profiles.mjs";

export const auditedPythonProjects = [
  "python",
  ...runtimeProfiles.map((profile) => `python/runtimes/${profile}`),
];

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  let failed = false;
  for (const project of auditedPythonProjects) {
    const auditExecution = spawnSync("uv", ["audit", "--project", project, "--locked"], {
      stdio: "inherit",
    });
    if (auditExecution.error) throw auditExecution.error;
    if (auditExecution.status !== 0) failed = true;
  }
  process.exitCode = failed ? 1 : 0;
}
