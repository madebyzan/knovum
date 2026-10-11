import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, realpathSync } from "node:fs";
import { join } from "node:path";

import { downloadArtifact } from "../transfer.mjs";

export async function buildWhisperLibraries(scratch, destination, target, configuration) {
  const archive = join(scratch, "whisper.tar.gz");
  await downloadArtifact(configuration.source, archive);
  const prefix = `whisper.cpp-${configuration.version}`;
  const names = execFileSync("tar", ["-tzf", archive], { encoding: "utf8", maxBuffer: 8 * 1024 ** 2 })
    .trim()
    .split("\n");
  if (
    names.some(
      (name) => name.split("/")[0] !== prefix || name.startsWith("/") || name.split("/").includes(".."),
    )
  )
    throw new Error("Invalid Whisper source archive");
  execFileSync("tar", ["-xzf", archive, "-C", scratch], { stdio: "inherit" });
  const source = join(scratch, prefix),
    build = join(scratch, "whisper-build");
  const flags = [
    "BUILD_SHARED_LIBS=ON",
    "GGML_BACKEND_DL=ON",
    "GGML_CPU_ALL_VARIANTS=ON",
    `GGML_METAL=${target === "macos-arm64" ? "ON" : "OFF"}`,
    "GGML_NATIVE=OFF",
    "GGML_OPENMP=OFF",
    "WHISPER_BUILD_TESTS=OFF",
    "WHISPER_BUILD_EXAMPLES=OFF",
    "WHISPER_BUILD_SERVER=OFF",
    ...["AVX", "AVX2", "FMA", "F16C", "BMI2"].map((name) => `GGML_${name}=OFF`),
  ];
  const args = ["-S", source, "-B", build, "-DCMAKE_BUILD_TYPE=Release", ...flags.map((flag) => `-D${flag}`)];
  if (target.startsWith("macos-"))
    args.push("-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0", "-DCMAKE_INSTALL_RPATH=@loader_path");
  execFileSync("cmake", args, { stdio: "inherit" });
  execFileSync("cmake", ["--build", build, "--config", "Release", "--parallel", "4"], {
    stdio: "inherit",
  });
  const extension = target === "windows-x64" ? ".dll" : ".dylib";
  const directory = join(destination, "compiled-libraries/whisper");
  mkdirSync(directory);
  const files = readdirSync(build, { recursive: true, withFileTypes: true })
    .filter(
      (entry) =>
        !entry.isDirectory() &&
        (entry.name.endsWith(extension) || (target.startsWith("macos-") && entry.name.endsWith(".so"))),
    )
    .map((entry) => join(entry.parentPath, entry.name));
  const namesSeen = new Set();
  for (const path of files) {
    const name = path.split(/[\\/]/u).at(-1);
    if (namesSeen.has(name)) continue;
    namesSeen.add(name);
    copyFileSync(realpathSync(path), join(directory, name));
  }
  if (![...namesSeen].some((name) => name.includes("whisper")))
    throw new Error("Whisper inference library is missing");
  const notices = join(destination, "source-notices/whisper");
  mkdirSync(notices, { recursive: true });
  copyFileSync(join(source, "LICENSE"), join(notices, "LICENSE"));
  // The frozen Whisper tree licenses its included GGML under the project-root MIT notice.
}
