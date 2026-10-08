"""Build pinned audio.cpp alignment and the application voice streaming ABI."""

from __future__ import annotations

import json
import shutil
import subprocess
import tarfile
from pathlib import Path, PurePosixPath
from typing import Any

from runtime.artifacts import digest, download, materialize_aliases
from runtime.commands import run


def prepare_metal_toolchain() -> None:
    probe = subprocess.run(
        ["xcrun", "--sdk", "macosx", "metal", "--version"],
        text=True,
        capture_output=True,
        check=False,
    )
    if probe.returncode == 0:
        return
    if "missing Metal Toolchain" not in probe.stderr:
        raise RuntimeError(f"Metal compiler discovery failed: {probe.stderr.strip()}")
    run("xcodebuild", "-downloadComponent", "MetalToolchain")
    run("xcrun", "--sdk", "macosx", "metal", "--version")


def vulkan_arguments(configuration: dict[str, Any], scratch: Path) -> list[str]:
    installer = scratch / configuration["fileName"]
    download(configuration, installer)
    sdk = scratch / "vulkan-sdk"
    run(
        str(installer),
        "--root",
        str(sdk),
        "--accept-licenses",
        "--default-answer",
        "--confirm-command",
        "install",
        "copy_only=1",
    )
    return [
        "-DENGINE_ENABLE_VULKAN=ON",
        f"-DVulkan_INCLUDE_DIR={sdk / 'Include'}",
        f"-DVulkan_LIBRARY={sdk / 'Lib/vulkan-1.lib'}",
        f"-DVulkan_GLSLC_EXECUTABLE={sdk / 'Bin/glslc.exe'}",
    ]


def install(project: Path, scratch: Path, runtime: Path, target: str) -> dict[str, Any]:
    configuration = json.loads((project / "runtime-source.json").read_text())["voiceNative"]
    version = run("cmake", "--version", capture=True).splitlines()[0].split()[-1]
    if version != configuration["cmakeVersion"]:
        raise ValueError("Native voice builder CMake differs from the frozen toolchain")
    source_archive = scratch / configuration["source"]["fileName"]
    download(configuration["source"], source_archive)
    source_root = f"audio.cpp-{configuration['revision']}"
    with tarfile.open(source_archive) as source_tar:
        for entry in source_tar.getmembers():
            path = PurePosixPath(entry.name)
            if path.is_absolute() or ".." in path.parts or path.parts[0] != source_root:
                raise ValueError("Native voice source archive contains an invalid path")
        source_tar.extractall(scratch, filter="data")
    supplier = scratch / source_root
    tools = project.parent / "scripts/runtime"
    build_project = scratch / "voice-build-project"
    build_project.mkdir()
    shutil.copy2(tools / "voice_bridge.cmake", build_project / "CMakeLists.txt")
    build = scratch / "voice-build"
    arguments = [
        "cmake",
        "-S",
        str(build_project),
        "-B",
        str(build),
        f"-DKNOVUM_AUDIOCPP_SOURCE={supplier}",
        f"-DKNOVUM_VOICE_BRIDGE={tools / 'voice_bridge.cpp'}",
        "-DCMAKE_BUILD_TYPE=Release",
        f"-DENGINE_ENABLE_METAL={'ON' if target.startswith('macos-') else 'OFF'}",
        "-DENGINE_ENABLE_CUDA=OFF",
        "-DENGINE_ENABLE_VULKAN=OFF",
        "-DGGML_AVX=OFF",
        "-DGGML_AVX2=OFF",
        "-DGGML_FMA=OFF",
        "-DGGML_F16C=OFF",
        "-DGGML_BMI2=OFF",
    ]
    if target.startswith("macos-"):
        prepare_metal_toolchain()
        desktop = json.loads((project.parent / "src-tauri/tauri.conf.json").read_text())
        arguments.append(f"-DCMAKE_OSX_DEPLOYMENT_TARGET={desktop['bundle']['macOS']['minimumSystemVersion']}")
        arguments.append(f"-DGGML_METAL_MACOSX_VERSION_MIN={desktop['bundle']['macOS']['minimumSystemVersion']}")
    if target == "windows-x64":
        arguments.extend(vulkan_arguments(configuration["vulkanSdk"], scratch))
    run(*arguments)
    run(
        "cmake",
        "--build",
        str(build),
        "--config",
        "Release",
        "--target",
        "knovum_voice",
        "audiocpp",
        "--parallel",
        "4",
    )
    run(
        "cmake",
        "--install",
        str(build),
        "--config",
        "Release",
        "--prefix",
        str(runtime),
        "--component",
        "knovum-voice",
    )
    materialize_aliases(runtime)
    notices = runtime / "native-notices" / "audio-cpp"
    notices.mkdir()
    for relative in (
        "LICENSE",
        "external/ggml/LICENSE",
        "external/sentencepiece/LICENSE",
        "external/cJSON/LICENSE",
        "external/libyaml/License",
        "external/sentencepiece/third_party/protobuf-lite/LICENSE",
        "external/sentencepiece/third_party/esaxx/LICENSE",
        "external/sentencepiece/third_party/absl/LICENSE",
        "external/sentencepiece/third_party/darts_clone/LICENSE",
    ):
        source = supplier / relative
        destination = notices / relative.replace("/", "-")
        shutil.copy2(source, destination)
    return {
        "name": "audio.cpp",
        "version": configuration["revision"],
        "license": configuration["license"],
        "files": [configuration["source"]],
        "cmakeVersion": version,
        "bridgeSha256": digest(tools / "voice_bridge.cpp"),
        "recipeSha256": digest(tools / "voice_bridge.cmake"),
        "sourceSha256": digest(source_archive),
        "portableCpu": True,
        "cpuDispatch": target in {"macos-x64", "windows-x64"},
        "vulkanSdk": configuration["vulkanSdk"] if target == "windows-x64" else None,
    }
