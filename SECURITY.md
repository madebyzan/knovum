# Security

Knovum is a desktop application with access to local repositories, team data, credentials, files, and OS capabilities. Security boundaries are part of the product architecture.

## Rules

- Grant the minimum Tauri capability and scope required by the current feature.
- Keep Shell, arbitrary filesystem, HTTP/WebSocket, updater, and custom command permissions closed until an explicit feature needs them.
- Store tokens and credentials only in the OS credential store. Never log them or persist them in Tauri Store, localStorage, SQLite, Pinia, or query caches.
- Validate all external input: deep links, files, network payloads, IPC payloads, persisted schema data, and user-entered paths/URLs.
- Keep CSP changes narrow and review every new origin.
- Do not bypass certificate/signature verification for production updates.
- Do not commit signing certificates, updater private keys, API secrets, private registry credentials, or local `.env` values.

## Review triggers

Any PR that changes capabilities, permissions, CSP, credentials, updater, Rust commands, file/network scopes, deep links, or window types requires explicit security review in the PR description.

## Reporting

For an internal deployment, report suspected vulnerabilities through the team's private security channel rather than a public issue. Include affected version, platform, reproduction steps, impact, and any relevant logs with secrets removed.
