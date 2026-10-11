import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { extractSelectedZip } from "./selected-zip.mjs";
import { downloadArtifact } from "./transfer.mjs";

export async function installStaticMedia(configuration, target, scratch, root) {
  const source = configuration.targets[target];
  const archive = join(scratch, "ffmpeg.zip");
  await downloadArtifact(source, archive);
  const executable = join(root, "bin", target === "windows-x64" ? "ffmpeg.exe" : "ffmpeg");
  const notices = join(root, "notices");
  mkdirSync(notices, { recursive: true });
  const selected = new Map([
    [source.entry, executable],
    ...Object.entries(source.archiveNotices ?? {}).map(([name, entry]) => [entry, join(notices, name)]),
  ]);
  await extractSelectedZip(archive, selected);
  chmodSync(executable, 0o755);
  await downloadArtifact(configuration.licenseSource, join(notices, "COPYING.GPLv3"));
  await Promise.all(
    (source.noticeSources ?? []).map((notice) => downloadArtifact(notice, join(notices, notice.fileName))),
  );
  writeFileSync(
    join(notices, "source-offer.json"),
    `${JSON.stringify({ license: configuration.license, ffmpegSource: configuration.sourceOffer, supplierBuild: source.buildSource, correspondingSourceReview: "required" }, null, 2)}\n`,
  );
  return executable;
}

export function dumpbinPath() {
  const installer = join(process.env["PROGRAMFILES(X86)"], "Microsoft Visual Studio/Installer/vswhere.exe");
  const output = execFileSync(
    installer,
    [
      "-latest",
      "-products",
      "*",
      "-requires",
      "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
      "-find",
      "VC/Tools/MSVC/*/bin/Hostx64/x64/dumpbin.exe",
    ],
    { encoding: "utf8" },
  );
  const paths = output
    .trim()
    .split(/\r?\n/u)
    .filter(Boolean)
    .toSorted((a, b) => b.localeCompare(a, "en", { numeric: true }));
  if (!paths.length) throw new Error("Windows media production requires MSVC build tools");
  return paths[0];
}

export function requireStaticMedia(executable, target, repository) {
  if (target.startsWith("macos-")) {
    const output = execFileSync("otool", ["-L", executable], { encoding: "utf8" });
    const dependencies = output
      .split("\n")
      .filter((line) => line.startsWith("\t"))
      .map((line) => line.trim().split(" (")[0]);
    if (
      !dependencies.length ||
      dependencies.some((name) => !name.startsWith("/usr/lib/") && !name.startsWith("/System/Library/"))
    )
      throw new Error("Static FFmpeg may depend only on macOS system libraries");
    return;
  }
  const allowed = new Set(
    JSON.parse(readFileSync(join(repository, "scripts/runtime/windows-system-libraries.json"), "utf8")),
  );
  const output = execFileSync(dumpbinPath(), ["/NOLOGO", "/DEPENDENTS", executable], {
    encoding: "utf8",
    env: { ...process.env, VSLANG: "1033" },
  });
  const dependencies = [...output.matchAll(/^\s+([\w.+-]+\.(?:dll|drv))\s*$/gim)].map((match) =>
    match[1].toLowerCase(),
  );
  const system = join(process.env.SYSTEMROOT, "System32");
  if (
    !dependencies.length ||
    dependencies.some(
      (name) =>
        !name.startsWith("api-ms-") &&
        !name.startsWith("ext-ms-") &&
        !(allowed.has(name) && existsSync(join(system, name))),
    )
  )
    throw new Error("Static FFmpeg may depend only on Windows system libraries");
}
