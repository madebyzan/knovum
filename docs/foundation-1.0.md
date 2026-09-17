# Foundation 1.0 freeze criteria

Business feature work may start against this RC. Freeze the foundation as 1.0 after the repository has committed dependency lockfiles and the CI matrix has passed on macOS and Windows.

## Framework-owned

- hidden-main application bootstrap and fatal startup recovery
- versioned preferences envelope + raw-file recovery reset
- theme, semantic typography, density and locale
- macOS application/menu-bar status localization and Windows Tray localization
- macOS Dock + menu-bar status lifecycle and Windows Tray behavior
- single-instance application lifecycle
- command palette and command shortcut dispatch
- app-level and feature-level Vue error isolation
- secure OS credential-store commands with Tauri ACL
- Task Center and Tauri Channel bridge
- window state/session restore and taskbar/Dock progress
- logging and runtime diagnostics
- current Tauri plugin registration limited to capabilities with real product callers
- Reka-based shared UI and Tailwind v4 semantic tokens
- unit/E2E/static architecture quality gates
- developer installation, QA packaging, release and troubleshooting documentation
- repository `AGENTS.md`, contribution and security contracts

## Feature-owned

- product-specific HTTP/WebSocket scopes
- source-control product engines and Shell scopes
- feature-specific editor, search or AI engines beyond the Space canvas engine
- feature SQLite schemas and migrations
- multi-window types
- file associations and external protocol handling
- provider-specific release channel policy and rollout UX beyond the installed signed updater foundation
- release signing and notarization

No speculative capabilities are granted for feature-owned items.
