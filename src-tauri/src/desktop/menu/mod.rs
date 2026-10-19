use tauri::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID},
    AppHandle, Emitter, Runtime,
};

use super::locale::{labels, MenuLabels};
use crate::preferences::LocalePreference;
mod state;

const COMMAND_EVENT: &str = "knovum-run-command";
const COMMAND_PALETTE_ID: &str = "shell.command-palette";
const SETTINGS_ID: &str = "navigate.settings";
const NEW_SPACE_ID: &str = "spaces.new-space";
const REOPEN_CLOSED_TAB_ID: &str = "spaces.reopen-closed-tab";
const CLOSE_TAB_ID: &str = "spaces.close-tab";
const FILE_MENU_ID: &str = "knovum.file";
const EDIT_MENU_ID: &str = "knovum.edit";
const UNDO_ID: &str = "editing.undo";
const REDO_ID: &str = "editing.redo";
const CUT_ID: &str = "editing.cut";
const COPY_ID: &str = "editing.copy";
const PASTE_ID: &str = "editing.paste";
const SELECT_ALL_ID: &str = "editing.select-all";
/// Every menu item is a renderer command, so the identifier is the whole contract between them.
const COMMAND_IDS: [&str; 11] = [
    COMMAND_PALETTE_ID,
    SETTINGS_ID,
    NEW_SPACE_ID,
    REOPEN_CLOSED_TAB_ID,
    CLOSE_TAB_ID,
    UNDO_ID,
    REDO_ID,
    CUT_ID,
    COPY_ID,
    PASTE_ID,
    SELECT_ALL_ID,
];

pub fn build<R: Runtime>(app: &AppHandle<R>, locale: LocalePreference) -> tauri::Result<Menu<R>> {
    let labels = labels(locale).map_err(std::io::Error::other)?.menu;
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

    let edit = edit_submenu(app, &labels)?;
    menu.remove_at(2)?;
    menu.insert(&edit, 2)?;

    Ok(menu)
}

/// macOS resolves the standard editing chords through the Application Menu, never inside the
/// webview, so every item owns its native accelerator and the renderer decides whether the focused
/// text control or the active surface performs it.
fn edit_submenu<R: Runtime>(app: &AppHandle<R>, labels: &MenuLabels) -> tauri::Result<Submenu<R>> {
    let item = |id: &str, text: &str, accelerator: &str| {
        MenuItem::with_id(app, id, text, true, Some(accelerator))
    };
    let undo = item(UNDO_ID, labels.undo, "CmdOrCtrl+Z")?;
    let redo = item(REDO_ID, labels.redo, "CmdOrCtrl+Shift+Z")?;
    let cut = item(CUT_ID, labels.cut, "CmdOrCtrl+X")?;
    let copy = item(COPY_ID, labels.copy, "CmdOrCtrl+C")?;
    let paste = item(PASTE_ID, labels.paste, "CmdOrCtrl+V")?;
    let select_all = item(SELECT_ALL_ID, labels.select_all, "CmdOrCtrl+A")?;
    Submenu::with_id_and_items(
        app,
        EDIT_MENU_ID,
        labels.edit,
        true,
        &[
            &undo,
            &redo,
            &PredefinedMenuItem::separator(app)?,
            &cut,
            &copy,
            &paste,
            &PredefinedMenuItem::separator(app)?,
            &select_all,
        ],
    )
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

pub fn rebuild<R: Runtime>(app: &AppHandle<R>, locale: LocalePreference) -> tauri::Result<()> {
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
    let command_id = event.id().as_ref();
    if !COMMAND_IDS.contains(&command_id) {
        return;
    }
    if let Err(error) = app.emit(COMMAND_EVENT, command_id) {
        log::error!("failed to handle application menu command: {error}");
    }
}
