# Troubleshooting

## Main window opens with bootstrap error

Use the displayed error details first. Preferences use a versioned schema. If the Store file itself is corrupted, use the explicit Reset Settings action; it removes application settings and saved provider credentials as one recoverable reset operation, while leaving feature databases and Space content intact.

## Global wake shortcut is unavailable

Another application may own `Cmd/Ctrl+Shift+Space`. This is an expected runtime outcome; the app remains usable through Dock/Taskbar/Tray. Do not fail the full application bootstrap for this condition.

## Windows installer build fails around WiX/light.exe

MSI packaging requires Windows tooling and may require the Windows VBSCRIPT optional feature. Run Windows packaging on a Windows host. NSIS is the primary interactive installer if MSI is not required for the current deployment.

## macOS package is reported as damaged/unverified

For internal QA, use the packaging workflow's ad-hoc signing. For production distribution, configure a real Developer ID certificate and notarization. Do not tell users to bypass Gatekeeper as the release strategy.

## Lockfile gate fails in CI

Generate and commit both:

```bash
pnpm install
cargo generate-lockfile --manifest-path src-tauri/Cargo.toml
```

Then commit `pnpm-lock.yaml` and `src-tauri/Cargo.lock`.

## A feature needs filesystem/network/Shell access

Do not grant a broad default capability. Define the exact paths/origins/commands needed by the feature, document the OSS/architecture choice, then add the narrow capability scope.

## Browser preview behaves differently from desktop

Browser-only runtime is not supported. Use `pnpm desktop:dev` so Tauri IPC, window APIs, system locale, capabilities, credential store and lifecycle behavior are present.

## A development database uses an obsolete baseline

SQLx installs the current Spaces v1 baseline before the repository pool becomes available. A
changed checksum means the local development database was created from an obsolete baseline.

Before the first public release, stop Knovum and remove the development `spaces.db` from Tauri's
AppConfig directory. Do not add a repair migration, compatibility reader, or parallel schema path.

For SQLite changes, `pnpm check:rust` exercises the embedded SQLx baseline against fresh
in-memory databases.
