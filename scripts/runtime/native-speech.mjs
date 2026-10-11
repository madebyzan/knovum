import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";

import { extractSourceZip } from "./native-source-zip.mjs";
import { downloadArtifact } from "./transfer.mjs";

export async function buildNativeSpeech(scratch, destination, target, configuration) {
  const extension = target === "windows-x64" ? ".dll" : ".dylib";
  const headers = join(scratch, "ort-headers");
  mkdirSync(headers);
  await Promise.all(
    configuration.headers.map((source) => downloadArtifact(source, join(headers, source.fileName))),
  );
  const ort = join(scratch, "ort-lib");
  mkdirSync(ort);
  copyFileSync(
    join(destination, target === "windows-x64" ? "onnxruntime.dll" : "libonnxruntime.1.22.1.dylib"),
    join(ort, target === "windows-x64" ? "onnxruntime.dll" : "libonnxruntime.dylib"),
  );
  if (target === "windows-x64") {
    const archive = join(scratch, "ort-windows.zip");
    await downloadArtifact(configuration.windowsImportLibrary, archive);
    const root = await extractSourceZip(archive, scratch, { prefix: "onnxruntime-win-x64-1.22.1" });
    copyFileSync(join(root, "lib/onnxruntime.lib"), join(ort, "onnxruntime.lib"));
  }
  const archive = join(scratch, "sherpa.zip");
  await downloadArtifact(configuration.source, archive);
  const source = await extractSourceZip(archive, scratch, {
    prefix: `sherpa-onnx-${configuration.revision}`,
    select: (path) =>
      path === "CMakeLists.txt" ||
      path === "LICENSE" ||
      path.startsWith("cmake/") ||
      path.startsWith("sherpa-onnx/"),
  });
  const build = join(scratch, "sherpa-build");
  const flags = [
    "BUILD_SHARED_LIBS=ON",
    "SHERPA_ONNX_ENABLE_PYTHON=OFF",
    "SHERPA_ONNX_ENABLE_TTS=OFF",
    "SHERPA_ONNX_ENABLE_PORTAUDIO=OFF",
    "SHERPA_ONNX_ENABLE_WEBSOCKET=OFF",
    "SHERPA_ONNX_ENABLE_BINARY=OFF",
    "SHERPA_ONNX_ENABLE_EXAMPLES=OFF",
    "SHERPA_ONNX_ENABLE_C_API=ON",
    "SHERPA_ONNX_ENABLE_SPEAKER_DIARIZATION=ON",
  ];
  configure(source, build, target, {
    flags,
    environment: { SHERPA_ONNXRUNTIME_INCLUDE_DIR: headers, SHERPA_ONNXRUNTIME_LIB_DIR: ort },
  });
  compile(build, ["sherpa-onnx-c-api"]);
  const candidates = findLibraries(build, extension).filter(
    (path) =>
      path.endsWith(`libsherpa-onnx-c-api${extension}`) || path.endsWith(`sherpa-onnx-c-api${extension}`),
  );
  if (candidates.length !== 1) throw new Error("Sherpa native library inventory differs");
  copyFileSync(candidates[0], join(destination, `compiled-libraries/sherpa-onnx-c-api${extension}`));
  const notices = join(destination, "source-notices/sherpa");
  mkdirSync(notices, { recursive: true });
  copyFileSync(join(source, "LICENSE"), join(notices, "LICENSE"));
  const dependencies = join(build, "_deps");
  for (const path of findNotices(dependencies)) {
    const output = join(notices, "dependencies", relative(dependencies, path));
    mkdirSync(dirname(output), { recursive: true });
    copyFileSync(path, output);
  }
}

function configure(source, build, target, { flags, environment }) {
  const args = ["-S", source, "-B", build, "-DCMAKE_BUILD_TYPE=Release", ...flags.map((flag) => `-D${flag}`)];
  if (target.startsWith("macos-"))
    args.push("-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0", "-DCMAKE_INSTALL_RPATH=@loader_path");
  execFileSync("cmake", args, { stdio: "inherit", env: { ...process.env, ...environment } });
}
function compile(build, targets) {
  execFileSync(
    "cmake",
    ["--build", build, "--config", "Release", "--target", ...targets, "--parallel", "4"],
    { stdio: "inherit" },
  );
}
function findLibraries(root, extension) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name);
    return entry.isDirectory()
      ? findLibraries(path, extension)
      : entry.name.endsWith(extension)
        ? [path]
        : [];
  });
}
function findNotices(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name);
    return entry.isDirectory()
      ? findNotices(path)
      : /^(LICENSE|COPYING|NOTICE)(\..*)?$/iu.test(entry.name) && statSync(path).size < 1024 ** 2
        ? [path]
        : [];
  });
}
