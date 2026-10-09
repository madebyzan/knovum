use tauri::{menu::Submenu, AppHandle, Runtime};

use super::{CLOSE_TAB_ID, FILE_MENU_ID, NEW_SPACE_ID, REOPEN_CLOSED_TAB_ID};

pub fn read<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<(bool, bool, bool)> {
    let file_menu = file_menu(app)?;
    let enabled = |id: &str| {
        file_menu
            .get(id)
            .and_then(|item| item.as_menuitem().cloned())
            .ok_or_else(|| std::io::Error::other(format!("File menu item is missing: {id}")))?
            .is_enabled()
    };
    Ok((
        enabled(NEW_SPACE_ID)?,
        enabled(CLOSE_TAB_ID)?,
        enabled(REOPEN_CLOSED_TAB_ID)?,
    ))
}

pub fn write<R: Runtime>(
    app: &AppHandle<R>,
    workspace_active: bool,
    can_close: bool,
    can_reopen: bool,
) -> tauri::Result<()> {
    let file_menu = file_menu(app)?;
    set_enabled(&file_menu, NEW_SPACE_ID, workspace_active)?;
    set_enabled(&file_menu, CLOSE_TAB_ID, workspace_active && can_close)?;
    set_enabled(
        &file_menu,
        REOPEN_CLOSED_TAB_ID,
        workspace_active && can_reopen,
    )
}

fn file_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    app.menu()
        .ok_or_else(|| std::io::Error::other("Knovum application menu is not installed"))?
        .get(FILE_MENU_ID)
        .and_then(|item| item.as_submenu().cloned())
        .ok_or_else(|| std::io::Error::other("Knovum File menu is not installed").into())
}

fn set_enabled<R: Runtime>(
    menu: &Submenu<R>,
    menu_item_id: &str,
    enabled: bool,
) -> tauri::Result<()> {
    menu.get(menu_item_id)
        .and_then(|item| item.as_menuitem().cloned())
        .ok_or_else(|| std::io::Error::other(format!("File menu item is missing: {menu_item_id}")))?
        .set_enabled(enabled)
}
