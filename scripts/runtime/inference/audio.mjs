import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { downloadArtifact } from "../transfer.mjs";

export async function buildAudioLibraries(repository, scratch, destination, { target, configuration }) {
  const version = execFileSync("cmake", ["--version"], { encoding: "utf8" }).split("\n")[0].split(" ").at(-1);
  if (version !== configuration.cmakeVersion)
    throw new Error("Inference audio CMake differs from the pinned toolchain");
  const archive = join(scratch, "audio-cpp-source.tar.gz");
  await downloadArtifact(configuration.source, archive);
  const root = `audio.cpp-${configuration.revision}`;
  const names = execFileSync("tar", ["-tzf", archive], { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 })
    .trim()
    .split("\n");
  if (
    names.some(
      (name) => name.split("/")[0] !== root || name.startsWith("/") || name.split("/").includes(".."),
    )
  )
    throw new Error("Inference audio source archive contains an invalid path");
  execFileSync("tar", ["-xzf", archive, "-C", scratch], { stdio: "inherit" });
  const project = join(scratch, "audio-project");
  mkdirSync(project);
  copyFileSync(
    join(repository, "scripts/runtime/inference/voice_bridge.cmake"),
    join(project, "CMakeLists.txt"),
  );
  const build = join(scratch, "audio-build");
  const args = [
    "-S",
    project,
    "-B",
    build,
    `-DKNOVUM_AUDIOCPP_SOURCE=${join(scratch, root)}`,
    `-DKNOVUM_VOICE_BRIDGE=${join(repository, "scripts/runtime/inference/voice_bridge.cpp")}`,
    "-DCMAKE_BUILD_TYPE=Release",
    `-DENGINE_ENABLE_METAL=${target.startsWith("macos-") ? "ON" : "OFF"}`,
    "-DENGINE_ENABLE_CUDA=OFF",
    "-DENGINE_ENABLE_VULKAN=OFF",
    ...["AVX", "AVX2", "FMA", "F16C", "BMI2"].map((name) => `-DGGML_${name}=OFF`),
  ];
  if (target.startsWith("macos-")) {
    const desktop = JSON.parse(readFileSync(join(repository, "src-tauri/tauri.conf.json"), "utf8"));
    args.push(
      `-DCMAKE_OSX_DEPLOYMENT_TARGET=${desktop.bundle.macOS.minimumSystemVersion}`,
      `-DGGML_METAL_MACOSX_VERSION_MIN=${desktop.bundle.macOS.minimumSystemVersion}`,
    );
  }
  if (target === "windows-x64") {
    const installer = join(scratch, configuration.vulkanSdk.fileName);
    await downloadArtifact(configuration.vulkanSdk, installer);
    const sdk = join(scratch, "vulkan-sdk");
    execFileSync(
      installer,
      ["--root", sdk, "--accept-licenses", "--default-answer", "--confirm-command", "install", "copy_only=1"],
      { stdio: "inherit" },
    );
    args.push(
      "-DENGINE_ENABLE_VULKAN=ON",
      `-DVulkan_INCLUDE_DIR=${join(sdk, "Include")}`,
      `-DVulkan_LIBRARY=${join(sdk, "Lib/vulkan-1.lib")}`,
      `-DVulkan_GLSLC_EXECUTABLE=${join(sdk, "Bin/glslc.exe")}`,
    );
  }
  execFileSync("cmake", args, { stdio: "inherit" });
  execFileSync(
    "cmake",
    ["--build", build, "--config", "Release", "--target", "knovum_voice", "audiocpp", "--parallel", "4"],
    { stdio: "inherit" },
  );
  execFileSync(
    "cmake",
    ["--install", build, "--config", "Release", "--prefix", destination, "--component", "knovum-voice"],
    { stdio: "inherit" },
  );
  const notices = join(destination, "source-notices/audio-cpp");
  mkdirSync(notices, { recursive: true });
  for (const path of [
    "LICENSE",
    "external/ggml/LICENSE",
    "external/sentencepiece/LICENSE",
    "external/cJSON/LICENSE",
    "external/libyaml/License",
  ]) {
    copyFileSync(join(scratch, root, path), join(notices, path.replaceAll("/", "-")));
  }
  // Vendor install aliases are materialized before the immutable inventory is sealed.
  for (const name of readdirSync(join(destination, "compiled-libraries"))) {
    if (!name.endsWith(target === "windows-x64" ? ".dll" : ".dylib"))
      throw new Error("Unexpected inference audio library");
  }
}
