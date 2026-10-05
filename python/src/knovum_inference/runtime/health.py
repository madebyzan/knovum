from __future__ import annotations

import platform

from knovum_inference.errors import server_error


def current_process_resident_bytes() -> int:
    system = platform.system()
    machine = platform.machine().lower()
    if system == "Darwin" and machine == "arm64":
        value = _macos_resident_bytes()
    elif system == "Windows" and machine in {"amd64", "x86_64"}:
        value = _windows_resident_bytes()
    else:
        raise server_error("unsupported_device", "Runtime health is unavailable on this platform")
    if value <= 0:
        raise server_error("runtime_mismatch", "Runtime resident memory is unavailable")
    return value


def _macos_resident_bytes() -> int:
    import ctypes

    class TimeValue(ctypes.Structure):
        _fields_ = [("seconds", ctypes.c_int32), ("microseconds", ctypes.c_int32)]

    class MachTaskBasicInfo(ctypes.Structure):
        _fields_ = [
            ("virtual_size", ctypes.c_uint64),
            ("resident_size", ctypes.c_uint64),
            ("resident_size_max", ctypes.c_uint64),
            ("user_time", TimeValue),
            ("system_time", TimeValue),
            ("policy", ctypes.c_int32),
            ("suspend_count", ctypes.c_int32),
        ]

    library = ctypes.CDLL("/usr/lib/libSystem.B.dylib")
    task_info = library.task_info
    task_info.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
    task_info.restype = ctypes.c_int
    task = ctypes.c_uint32.in_dll(library, "mach_task_self_").value
    info = MachTaskBasicInfo()
    count = ctypes.c_uint32(ctypes.sizeof(info) // ctypes.sizeof(ctypes.c_uint32))
    status = task_info(task, 20, ctypes.byref(info), ctypes.byref(count))
    if status != 0:
        raise server_error("runtime_mismatch", f"macOS task_info failed with status {status}")
    return int(info.resident_size)


def _windows_resident_bytes() -> int:
    import ctypes
    from ctypes import wintypes

    class ProcessMemoryCounters(ctypes.Structure):
        _fields_ = [
            ("cb", wintypes.DWORD),
            ("page_fault_count", wintypes.DWORD),
            ("peak_working_set_size", ctypes.c_size_t),
            ("working_set_size", ctypes.c_size_t),
            ("quota_peak_paged_pool_usage", ctypes.c_size_t),
            ("quota_paged_pool_usage", ctypes.c_size_t),
            ("quota_peak_non_paged_pool_usage", ctypes.c_size_t),
            ("quota_non_paged_pool_usage", ctypes.c_size_t),
            ("pagefile_usage", ctypes.c_size_t),
            ("peak_pagefile_usage", ctypes.c_size_t),
        ]

    win_dll = vars(ctypes)["WinDLL"]
    kernel32 = win_dll("kernel32", use_last_error=True)
    psapi = win_dll("psapi", use_last_error=True)
    kernel32.GetCurrentProcess.restype = wintypes.HANDLE
    psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.c_void_p, wintypes.DWORD]
    psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
    counters = ProcessMemoryCounters()
    counters.cb = ctypes.sizeof(counters)
    if not psapi.GetProcessMemoryInfo(
        kernel32.GetCurrentProcess(),
        ctypes.byref(counters),
        counters.cb,
    ):
        error = int(vars(ctypes)["get_last_error"]())
        raise server_error("runtime_mismatch", f"Windows GetProcessMemoryInfo failed with error {error}")
    return int(counters.working_set_size)
