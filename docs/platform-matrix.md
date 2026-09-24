# Desktop platform matrix

This is the framework-level contract for macOS and Windows. Business features should not reimplement these behaviors.

| Concern                    | macOS                                                                      | Windows                                                          | Foundation                                                                         |
| -------------------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Startup                    | Dock launch feedback; main appears when usable                             | Taskbar launch feedback; main appears when usable                | main starts hidden until usable/recoverable bootstrap state                        |
| Window chrome              | native traffic lights + overlay titlebar/material                          | native caption + Mica content material                           | platform-specific Tauri config                                                     |
| Leading pane chrome        | unified toolbar starts after traffic lights; pane splitters begin below it | ordinary pane geometry; native caption stays untouched           | shell-owned pane/titlebar contract                                                 |
| Theme                      | follows system or explicit light/dark                                      | follows system or explicit light/dark                            | configured + live OS listener                                                      |
| Window state               | size/position/maximized/fullscreen                                         | size/position/maximized/fullscreen                               | persisted                                                                          |
| Close button               | hides main window; app remains available from Dock/menu-bar status item    | hides main window to the system Tray                             | Rust-owned desktop lifecycle                                                       |
| Explicit quit              | native Application Menu / menu-bar status item                             | Windows system Tray                                              | Rust-owned                                                                         |
| Reopen                     | Dock/status item restores or focuses main                                  | Tray focuses main window while running                           | Rust-owned                                                                         |
| Command palette            | Cmd+K via native Application Menu                                          | Ctrl+K via WebView shortcut                                      | shared command registry + command palette                                          |
| Shortcut display           | symbols                                                                    | text modifiers                                                   | `UiKbd`                                                                            |
| Tray / status item         | installed as a menu-bar status item with app actions                       | installed as system Tray with app actions                        | shared Rust-owned desktop menu                                                     |
| Task progress              | Dock progress                                                              | taskbar progress                                                 | mirrored from Task Center                                                          |
| System dialogs             | macOS picker/dialog                                                        | Windows picker/dialog                                            | official dialog plugin ready                                                       |
| Spaces clipboard images    | native clipboard image read -> Rust-managed private AppData PNG            | native clipboard image read -> Rust-managed private AppData PNG  | feature-scoped clipboard read + Rust persistence                                   |
| Spaces file/folder drop    | Finder drag -> validated private resource copy                             | Explorer drag -> validated private resource copy                 | Tauri drag paths; no persisted external scope                                      |
| Spaces document thumbnails | system Quick Look page render cached as a private PNG                      | Windows Shell provider render cached as a private PNG            | one system-rendering policy; unavailable thumbnails keep the resource presentation |
| Spaces system preview      | spacebar or canvas menu opens the system panel for one private asset       | command answers with an explicit error; the menu omits the entry | shared foundation wrapper over platform preview adapters                           |
| Autostart                  | LaunchAgent                                                                | platform autostart                                               | Settings                                                                           |
| Locale                     | OS BCP-47 locale                                                           | OS BCP-47 locale                                                 | system/en/zh-CN mapping                                                            |
| Font stack                 | system/SF family                                                           | Segoe UI Variable                                                | tokenized                                                                          |
| Generated text             | PingFang SC via Core Text                                                  | Microsoft YaHei UI via DirectWrite                               | installed system fonts; no packaged font file                                      |
| Release trust              | signing + notarization                                                     | code-signed installer recommended                                | release configuration, credentials external                                        |

## Scope-dependent integrations

Document visual processing uses background Core Graphics PDF rendering on macOS and
Windows.Data.Pdf on Windows. No PDF renderer binary or rendering IPC is shipped.
This is separate from system document thumbnails and does not create a custom reader.
Other platforms return an explicit unsupported error. Native rendering/threading must
be manually validated on both supported desktop platforms.

These are intentionally not globally enabled because the correct configuration cannot be known before the owning feature exists:

- file associations / Open With: requires real extensions and ownership semantics;
- Shell: requires the Git/tool command and exact argument scope;
- filesystem writes outside app storage: requires a user-selected path or explicit feature scope;
- Tauri HTTP/WebSocket: requires known hosts/endpoints;
- updater endpoint/public key: requires the real release channel and signing keys;
- camera/microphone/screen capture entitlements: only when a real feature needs them;
- secondary windows: requires a real window type, lifetime and capability set.

Installing a Tauri plugin does not grant the WebView permission to use it. Capability changes accompany the feature that needs the permission.

## QA rule

Any change to lifecycle, window, menu, keyboard, file, notification, updater or process behavior must be manually verified on both macOS and Windows in addition to CI compile checks.
