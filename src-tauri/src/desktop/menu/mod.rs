use tauri::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID},
    AppHandle, Emitter, Runtime,
};

use self::labels::labels;

mod labels;
mod state;

const COMMAND_EVENT: &str = "knovum-run-command";
const COMMAND_PALETTE_ID: &str = "shell.command-palette";
const SETTINGS_ID: &str = "navigate.settings";
const NEW_SPACE_ID: &str = "spaces.new-space";
const REOPEN_CLOSED_TAB_ID: &str = "spaces.reopen-closed-tab";
const CLOSE_TAB_ID: &str = "spaces.close-tab";
const FILE_MENU_ID: &str = "knovum.file";

pub fn build<R: Runtime>(app: &AppHandle<R>, locale: &str) -> tauri::Result<Menu<R>> {
    let labels = labels(locale);
    let menu = Menu::default(app)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, labels.settings, true, Some("CmdOrCtrl+,"))?;
    let command_palette = MenuItem::with_id(
        app,
        COMMAND_PALETTE_ID,
        labels.command_palette,
        true,
        Some("CmdOrCtrl+K"),
    )?;

    let default_items = menu.items()?;
    if let Some(app_menu) = default_items.first() {
        if let Some(submenu) = app_menu.as_submenu() {
            submenu.insert(&settings, 2)?;
            submenu.insert(&command_palette, 3)?;
        }
    }

    let new_space = MenuItem::with_id(
        app,
        NEW_SPACE_ID,
        labels.new_space,
        true,
        Some("CmdOrCtrl+T"),
    )?;
    let reopen_closed_tab = MenuItem::with_id(
        app,
        REOPEN_CLOSED_TAB_ID,
        labels.reopen_closed_tab,
        true,
        Some("CmdOrCtrl+Shift+T"),
    )?;
    let file_separator = PredefinedMenuItem::separator(app)?;
    let close_tab = MenuItem::with_id(
        app,
        CLOSE_TAB_ID,
        labels.close_tab,
        true,
        Some("CmdOrCtrl+W"),
    )?;
    let file_submenu = Submenu::with_id_and_items(
        app,
        FILE_MENU_ID,
        labels.file,
        true,
        &[&new_space, &reopen_closed_tab, &file_separator, &close_tab],
    )?;
    menu.remove_at(1)?;
    menu.insert(&file_submenu, 1)?;
    remove_window_close_item(&menu)?;

    let cut = MenuItem::with_id(app, "editing.cut", labels.cut, true, None::<&str>)?;
    let copy = MenuItem::with_id(app, "editing.copy", labels.copy, true, None::<&str>)?;
    let paste = MenuItem::with_id(app, "editing.paste", labels.paste, true, None::<&str>)?;
    let edit = Submenu::with_id_and_items(
        app,
        "knovum.edit",
        labels.edit,
        true,
        &[&cut, &copy, &paste],
    )?;
    menu.remove_at(2)?;
    menu.insert(&edit, 2)?;

    Ok(menu)
}

fn remove_window_close_item<R: Runtime>(menu: &Menu<R>) -> tauri::Result<()> {
    let window_menu = menu
        .get(WINDOW_SUBMENU_ID)
        .and_then(|item| item.as_submenu().cloned())
        .ok_or_else(|| std::io::Error::other("Native Window menu is not installed"))?;
    let items = window_menu.items()?;
    let close_index = items
        .len()
        .checked_sub(1)
        .filter(|index| items[*index].as_predefined_menuitem().is_some())
        .ok_or_else(|| std::io::Error::other("Native Close Window menu item is missing"))?;
    window_menu.remove_at(close_index)?;
    Ok(())
}

pub fn rebuild<R: Runtime>(app: &AppHandle<R>, locale: &str) -> tauri::Result<()> {
    let workspace_state = state::read(app)?;
    app.set_menu(build(app, locale)?)?;
    state::write(app, workspace_state.0, workspace_state.1, workspace_state.2)?;
    Ok(())
}

pub fn set_workspace_state<R: Runtime>(
    app: &AppHandle<R>,
    workspace_active: bool,
    can_close: bool,
    can_reopen: bool,
) -> tauri::Result<()> {
    state::write(app, workspace_active, can_close, can_reopen)
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let handled = match event.id().as_ref() {
        id @ (COMMAND_PALETTE_ID | SETTINGS_ID | NEW_SPACE_ID | REOPEN_CLOSED_TAB_ID
        | CLOSE_TAB_ID | "editing.cut" | "editing.copy" | "editing.paste") => app
            .emit(COMMAND_EVENT, id)
            .map_err(|error| error.to_string()),
        _ => Ok(()),
    };
    if let Err(error) = handled {
        log::error!("failed to handle application menu command: {error}");
    }
}
