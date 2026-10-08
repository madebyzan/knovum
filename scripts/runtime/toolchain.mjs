import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, renameSync, rmSync } from "node:fs";
import { join } from "node:path";

import { sha256File } from "./files.mjs";
import { downloadArtifact } from "./transfer.mjs";

export const ensureUv = (repository, target, source) => ensureTool(repository, target, source, "uv");
export const ensureMicromamba = (repository, target, source) =>
  ensureTool(repository, target, source, "micromamba");

async function ensureTool(repository, target, source, tool) {
  const name = target === "windows-x64" ? `${tool}.exe` : tool;
  const parent = join(repository, ".runtime-development", "tools", target);
  const destination = join(parent, source.sha256);
  const executable = join(destination, name);
  if (existsSync(destination)) {
    checkExecutable(executable, source);
    return executable;
  }
  mkdirSync(parent, { recursive: true });
  const scratch = mkdtempSync(join(parent, ".installing-"));
  try {
    const archive = await downloadArtifact(
      source,
      join(scratch, tool === "micromamba" ? name : target === "windows-x64" ? "uv.zip" : "uv.tar.gz"),
    );
    if (tool === "micromamba") {
      if (target !== "windows-x64") chmodSync(archive, 0o755);
    } else if (target === "windows-x64") {
      execFileSync(
        "powershell.exe",
        [
          "-NoProfile",
          "-NonInteractive",
          "-Command",
          "Expand-Archive -LiteralPath $env:KNOVUM_UV_ARCHIVE -DestinationPath $env:KNOVUM_UV_STAGING",
        ],
        {
          env: { ...process.env, KNOVUM_UV_ARCHIVE: archive, KNOVUM_UV_STAGING: scratch },
          stdio: "inherit",
        },
      );
    } else {
      execFileSync("tar", ["-xzf", archive, "-C", scratch, source.entry], { stdio: "inherit" });
    }
    const extracted = tool === "micromamba" ? archive : join(scratch, source.entry);
    checkExecutable(extracted, source);
    const prepared = join(scratch, "prepared");
    mkdirSync(prepared);
    renameSync(extracted, join(prepared, name));
    renameSync(prepared, destination);
    return executable;
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

function checkExecutable(file, source) {
  if (sha256File(file) !== source.executableSha256)
    throw new Error("Managed tool executable digest differs from the pinned release");
}
