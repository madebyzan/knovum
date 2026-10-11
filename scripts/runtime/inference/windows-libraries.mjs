import { execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { listRuntimeFiles, sha256File } from "../files.mjs";
import { extractSelectedZip } from "../selected-zip.mjs";
import { downloadArtifact } from "../transfer.mjs";
import { isMicrosoftRuntime, verifyMicrosoftRuntime } from "../windows-runtime-signature.mjs";

export async function installWindowsInferenceLibraries(directory, repository) {
  const scratch = mkdtempSync(join(tmpdir(), "knovum-windows-runtime-"));
  try {
    installMicrosoftRuntime(scratch, directory, repository);
    const source = JSON.parse(
      readFileSync(join(repository, "scripts/runtime/inference/windows-source.json"), "utf8"),
    );
    const archive = join(scratch, source.vulkanLoader.fileName);
    await downloadArtifact(source.vulkanLoader, archive);
    const notices = join(directory, "source-notices/windows-runtime");
    mkdirSync(notices, { recursive: true });
    await extractSelectedZip(
      archive,
      new Map([
        [source.vulkanLoader.entry, join(directory, "bin/vulkan-1.dll")],
        [source.vulkanLoader.licenseEntry, join(notices, "VULKAN-LOADER-LICENSE.txt")],
      ]),
    );
    writeFileSync(
      join(notices, "MSVC-RUNTIME.txt"),
      "Microsoft Visual C++ v14 runtime: unmodified Microsoft-signed redistributable files selected by CMake InstallRequiredSystemLibraries from the licensed MSVC build toolchain. Distribution is subject to Microsoft Software License Terms and requires release review.\nhttps://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files\nhttps://aka.ms/VCRedistLicense\n",
    );
    mkdirSync(join(directory, "build-evidence"), { recursive: true });
    writeFileSync(join(directory, "build-evidence/windows-inference-source.json"), JSON.stringify(source));
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

function installMicrosoftRuntime(scratch, directory, repository) {
  const project = join(scratch, "msvc");
  mkdirSync(project);
  copyFileSync(
    join(repository, "scripts/runtime/inference/windows-runtime.cmake"),
    join(project, "CMakeLists.txt"),
  );
  const build = join(scratch, "build");
  execFileSync("cmake", ["-S", project, "-B", build, "-A", "x64", `-DCMAKE_INSTALL_PREFIX=${directory}`], {
    stdio: "inherit",
  });
  execFileSync("cmake", ["--install", build, "--config", "Release"], { stdio: "inherit" });
  const libraries = listRuntimeFiles(join(directory, "bin")).filter(isMicrosoftRuntime);
  if (!libraries.length) throw new Error("MSVC runtime installation produced no redistributable DLLs");
  const files = libraries.map((path) => {
    const file = join(directory, "bin", path);
    verifyMicrosoftRuntime(file);
    return { path: `bin/${path}`, sha256: sha256File(file) };
  });
  mkdirSync(join(directory, "build-evidence"), { recursive: true });
  copyFileSync(join(project, "CMakeLists.txt"), join(directory, "build-evidence/windows-runtime.cmake"));
  writeFileSync(
    join(directory, "build-evidence/msvc-runtime.json"),
    JSON.stringify({
      supplier: "MSVC licensed build toolchain",
      mechanism: "CMake InstallRequiredSystemLibraries",
      compiler: JSON.parse(readFileSync(join(build, "compiler.json"), "utf8")),
      files,
    }),
  );
}
