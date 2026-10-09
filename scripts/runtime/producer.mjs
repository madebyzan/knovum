import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { hostRuntimeTarget } from "./profiles.mjs";
import { ensureMicromamba, ensureUv } from "./toolchain.mjs";

export async function runProducer(repository, args, entry = "build_python.py") {
  const target = hostRuntimeTarget();
  const configuration = JSON.parse(readFileSync(join(repository, "python/runtime-source.json"), "utf8"));
  const uv = await ensureUv(repository, target, configuration.uv.targets[target]);
  const micromamba =
    entry === "build_python.py"
      ? await ensureMicromamba(repository, target, configuration.micromamba.targets[target])
      : undefined;
  execFileSync(
    uv,
    [
      "run",
      "--project",
      join(repository, "python"),
      "--only-group",
      "publication",
      "--frozen",
      "python",
      "-I",
      "-B",
      "-X",
      "utf8",
      join(repository, "scripts/runtime", entry),
      ...args,
    ],
    {
      cwd: repository,
      env: {
        ...process.env,
        PATH: [dirname(uv), ...(micromamba ? [dirname(micromamba)] : []), process.env.PATH ?? ""].join(
          delimiter,
        ),
      },
      stdio: "inherit",
    },
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  await runProducer(repository, process.argv.slice(2));
}
