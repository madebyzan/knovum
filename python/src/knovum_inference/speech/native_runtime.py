"""Load native audio engines exclusively from the verified worker environment."""

from __future__ import annotations

import ctypes
import locale
import os
import sys
from pathlib import Path
from typing import Any

from knovum_inference.errors import server_error


def load_native_library(stem: str) -> Any:
    suffix = {"darwin": ".dylib", "win32": ".dll"}.get(sys.platform)
    if suffix is None:
        raise server_error("unsupported_device", "Native audio requires macOS or Windows")
    # GGML discovers dispatch kernels and plugins from these controlled locations.
    locale.setlocale(locale.LC_CTYPE, "C")
    backend_directory = Path(sys.prefix) / "bin"
    os.chdir(backend_directory)
    os.environ.pop("GGML_BACKEND_PATH", None)
    dll_directory: Any = None
    if sys.platform == "win32":
        dll_directory = os.add_dll_directory(str(backend_directory))
    try:
        library: Any = ctypes.CDLL(str(Path(sys.prefix) / "native" / f"{stem}{suffix}"))
    except OSError as error:
        raise server_error("runtime_mismatch", "Native audio supplier is unavailable") from error
    library._knovum_dll_directory = dll_directory
    return library
