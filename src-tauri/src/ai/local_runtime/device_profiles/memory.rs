use super::MemorySnapshot;

#[cfg(target_os = "macos")]
use std::sync::OnceLock;

#[cfg(target_os = "macos")]
pub(super) fn physical_memory() -> Result<u64, String> {
    let bytes = objc2_foundation::NSProcessInfo::processInfo().physicalMemory();
    if bytes == 0 {
        return Err("resource_exhausted: Physical memory could not be determined".into());
    }
    Ok(bytes)
}

#[cfg(target_os = "macos")]
pub(super) fn platform_memory_snapshot() -> Result<MemorySnapshot, String> {
    #[repr(C, align(8))]
    #[derive(Default)]
    struct VmStatistics64 {
        free_count: u32,
        _active_count: u32,
        inactive_count: u32,
        _wire_count: u32,
        _zero_fill_count: u64,
        _reactivations: u64,
        _pageins: u64,
        _pageouts: u64,
        _faults: u64,
        _cow_faults: u64,
        _lookups: u64,
        _hits: u64,
        _purges: u64,
        _purgeable_count: u32,
        _speculative_count: u32,
        _decompressions: u64,
        _compressions: u64,
        _swapins: u64,
        _swapouts: u64,
        _compressor_page_count: u32,
        _throttled_count: u32,
        _external_page_count: u32,
        _internal_page_count: u32,
        _total_uncompressed_pages_in_compressor: u64,
        _swapped_count: u64,
    }
    unsafe extern "C" {
        fn mach_host_self() -> u32;
        fn host_page_size(host: u32, page_size: *mut u32) -> i32;
        fn host_statistics64(host: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
    }
    static HOST_PAGE: OnceLock<Result<(u32, u32), String>> = OnceLock::new();
    let &(host, page_size) = HOST_PAGE
        .get_or_init(|| {
            let host = unsafe { mach_host_self() };
            let mut page_size = 0_u32;
            let status = unsafe { host_page_size(host, &mut page_size) };
            if host == 0 || status != 0 || page_size == 0 {
                return Err("resource_exhausted: macOS memory page size is unavailable".into());
            }
            Ok((host, page_size))
        })
        .as_ref()
        .map_err(|error| error.clone())?;
    let mut statistics = VmStatistics64::default();
    let mut count =
        u32::try_from(std::mem::size_of::<VmStatistics64>() / std::mem::size_of::<i32>()).map_err(
            |error| format!("resource_exhausted: macOS memory statistics size is invalid: {error}"),
        )?;
    let status = unsafe {
        host_statistics64(
            host,
            4,
            (&mut statistics as *mut VmStatistics64).cast::<i32>(),
            &mut count,
        )
    };
    if status != 0 {
        return Err(format!(
            "resource_exhausted: macOS available memory could not be determined ({status})"
        ));
    }
    let available_pages = u64::from(statistics.free_count)
        .checked_add(u64::from(statistics.inactive_count))
        .ok_or("resource_exhausted: macOS available page count overflowed")?;
    let available_bytes = available_pages
        .checked_mul(u64::from(page_size))
        .ok_or("resource_exhausted: macOS available memory overflowed")?;
    Ok(MemorySnapshot {
        total_bytes: physical_memory()?,
        available_bytes,
    })
}

#[cfg(target_os = "windows")]
pub(super) fn physical_memory() -> Result<u64, String> {
    use windows::Win32::System::SystemInformation::GetPhysicallyInstalledSystemMemory;
    let mut installed_kib = 0_u64;
    unsafe { GetPhysicallyInstalledSystemMemory(&mut installed_kib) }.map_err(|error| {
        format!("resource_exhausted: Physical memory could not be determined: {error}")
    })?;
    if installed_kib == 0 {
        return Err("resource_exhausted: Physical memory is unavailable".into());
    }
    installed_kib
        .checked_mul(1024)
        .ok_or("resource_exhausted: Physical memory exceeds platform range".into())
}

#[cfg(target_os = "windows")]
pub(super) fn platform_memory_snapshot() -> Result<MemorySnapshot, String> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut status) }.map_err(|error| {
        format!("resource_exhausted: Available memory could not be determined: {error}")
    })?;
    Ok(MemorySnapshot {
        total_bytes: status.ullTotalPhys,
        available_bytes: status.ullAvailPhys,
    })
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn physical_memory() -> Result<u64, String> {
    Err("unsupported_device: Local inference is not qualified for this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn platform_memory_snapshot() -> Result<MemorySnapshot, String> {
    Err("unsupported_device: Local inference is not qualified for this platform".into())
}
