use std::collections::HashSet;

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
            HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_SAM_FLAGS,
            RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ,
        },
    },
};

use crate::ai::{
    contracts::AgentKind,
    error::{AiError, AiResult},
};

const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

pub(crate) fn detect() -> AiResult<HashSet<AgentKind>> {
    let mut installed = HashSet::new();
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let Some(root) = open(hive, UNINSTALL_KEY, view)? else {
                continue;
            };
            let mut index = 0;
            loop {
                // Windows limits registry key names to 255 UTF-16 characters.
                let mut name = [0u16; 256];
                let mut length = name.len() as u32;
                let status = unsafe {
                    RegEnumKeyExW(
                        root.0,
                        index,
                        Some(PWSTR(name.as_mut_ptr())),
                        &mut length,
                        None,
                        None,
                        None,
                        None,
                    )
                };
                if status == ERROR_NO_MORE_ITEMS {
                    break;
                }
                status.ok().map_err(registry_error)?;
                let key = String::from_utf16(&name[..length as usize])
                    .map_err(|error| AiError::internal(error.to_string()))?;
                if let Some(kind) = installed_agent(root.0, &key)? {
                    installed.insert(kind);
                }
                index += 1;
            }
        }
    }
    Ok(installed)
}

fn installed_agent(hive: HKEY, key: &str) -> AiResult<Option<AgentKind>> {
    let Some(name) = read_string(hive, key, "DisplayName")? else {
        return Ok(None);
    };
    let agent = super::super::catalog::AGENTS.iter().find(|agent| {
        agent
            .desktop_client
            .as_ref()
            .is_some_and(|client_installation| {
                client_installation.display_names.contains(&name.as_str())
            })
    });
    let Some(agent) = agent else { return Ok(None) };
    // InstallLocation is optional in Windows application registration. When
    // supplied, reject stale registrations whose installation directory is gone.
    let location = read_string(hive, key, "InstallLocation")?;
    Ok(location
        .filter(|path| !path.is_empty())
        .is_none_or(|path| std::path::Path::new(&path).is_dir())
        .then_some(agent.descriptor.kind))
}

fn open(hive: HKEY, path: &str, view: REG_SAM_FLAGS) -> AiResult<Option<RegistryKey>> {
    let path = utf16(path);
    let mut key = HKEY::default();
    let status =
        unsafe { RegOpenKeyExW(hive, PCWSTR(path.as_ptr()), None, KEY_READ | view, &mut key) };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    status.ok().map_err(registry_error)?;
    Ok(Some(RegistryKey(key)))
}

fn read_string(hive: HKEY, key: &str, value: &str) -> AiResult<Option<String>> {
    let key = utf16(key);
    let value = utf16(value);
    let flags = RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ;
    let mut bytes = 0;
    let status = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            flags,
            None,
            None,
            Some(&mut bytes),
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    status.ok().map_err(registry_error)?;
    let mut data = vec![0u16; (bytes as usize).div_ceil(2)];
    let status = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            flags,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    status.ok().map_err(registry_error)?;
    let length = data
        .iter()
        .position(|unit| *unit == 0)
        .ok_or_else(|| AiError::internal("Registry string is not null-terminated"))?;
    String::from_utf16(&data[..length])
        .map(Some)
        .map_err(|error| AiError::internal(error.to_string()))
}

fn utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn registry_error(error: windows::core::Error) -> AiError {
    AiError::internal(error.to_string())
}
