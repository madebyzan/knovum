use std::{ffi::OsString, path::Path};

use windows::Win32::UI::Shell::PropertiesSystem::IInitializeWithStream;
use windows::{
    core::{Interface, GUID, HSTRING, PCWSTR, PWSTR},
    Win32::{
        System::Com::{CLSIDFromString, CoCreateInstance, CLSCTX_INPROC_SERVER, STGM_READ},
        UI::Shell::{
            AssocQueryStringW, IInitializeWithItem, IPreviewHandler, IShellItem,
            PropertiesSystem::IInitializeWithFile, SHCreateItemFromParsingName,
            SHCreateStreamOnFileW, ASSOCF_NONE, ASSOCSTR_SHELLEXTENSION,
        },
    },
};

const PREVIEW_HANDLER_IID: &str = "{8895b1c6-b41f-4c1c-a562-0d564250836f}";

pub(super) fn load(source: &Path) -> Result<IPreviewHandler, String> {
    let class = registered_handler(source)?;
    let handler: IPreviewHandler = unsafe { CoCreateInstance(&class, None, CLSCTX_INPROC_SERVER) }
        .map_err(|error| {
            format!("Windows could not start the registered preview handler: {error}")
        })?;
    initialize(&handler, source)?;
    Ok(handler)
}

fn registered_handler(source: &Path) -> Result<GUID, String> {
    let extension = source
        .extension()
        .ok_or("This document has no file extension for Windows preview association")?;
    let mut association = OsString::from(".");
    association.push(extension);
    let association = HSTRING::from(association);
    let interface = HSTRING::from(PREVIEW_HANDLER_IID);
    let mut length = 0;
    let first = unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_SHELLEXTENSION,
            &association,
            &interface,
            None,
            &mut length,
        )
    };
    if length == 0 {
        return Err(format!(
            "No Windows preview handler is registered for this document (0x{:08X})",
            first.0 as u32
        ));
    }
    let mut class = vec![0_u16; length as usize];
    unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_SHELLEXTENSION,
            &association,
            &interface,
            Some(PWSTR(class.as_mut_ptr())),
            &mut length,
        )
    }
    .ok()
    .map_err(|error| format!("Windows could not read the preview association: {error}"))?;
    unsafe { CLSIDFromString(PCWSTR(class.as_ptr())) }
        .map_err(|error| format!("Windows returned an invalid preview handler identifier: {error}"))
}

fn initialize(handler: &IPreviewHandler, source: &Path) -> Result<(), String> {
    let path = HSTRING::from(source.as_os_str());
    if let Ok(initializer) = handler.cast::<IInitializeWithStream>() {
        let stream = unsafe { SHCreateStreamOnFileW(&path, STGM_READ.0) }
            .map_err(|error| format!("Windows could not open the document stream: {error}"))?;
        return unsafe { initializer.Initialize(&stream, STGM_READ.0) }
            .map_err(|error| format!("The preview handler rejected the document stream: {error}"));
    }
    let item: IShellItem = unsafe { SHCreateItemFromParsingName(&path, None) }
        .map_err(|error| format!("Windows Shell could not open this document: {error}"))?;
    if let Ok(initializer) = handler.cast::<IInitializeWithItem>() {
        return unsafe { initializer.Initialize(&item, STGM_READ.0) }
            .map_err(|error| format!("The preview handler rejected the Shell item: {error}"));
    }
    if let Ok(initializer) = handler.cast::<IInitializeWithFile>() {
        return unsafe { initializer.Initialize(&path, STGM_READ.0) }
            .map_err(|error| format!("The preview handler rejected this file: {error}"));
    }
    Err("The registered preview handler exposes no supported initialization interface".into())
}
