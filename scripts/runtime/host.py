"""Only supported native builders may produce desktop resources."""

import platform


def host_target() -> str:
    machine = platform.machine().lower()
    if platform.system() == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if platform.system() == "Darwin" and machine == "x86_64":
        return "macos-x64"
    if platform.system() == "Windows" and machine in {"amd64", "x86_64"}:
        return "windows-x64"
    raise ValueError(f"Unsupported native runtime builder: {platform.system()}/{machine}")
