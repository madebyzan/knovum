# Troubleshooting

## Main window opens with bootstrap error

Use the displayed error details first. Preferences use a versioned schema. If the Store file itself is corrupted, use the explicit Reset Settings action; it removes application settings and saved provider credentials as one recoverable reset operation, while leaving feature databases and Space content intact.

## Global wake shortcut is unavailable

Another application may own `Cmd/Ctrl+Shift+Space`. This is an expected runtime outcome; the app remains usable through Dock/Taskbar/Tray. Do not fail the full application bootstrap for this condition.

## Windows inference build fails in a nested supplier CMake project

MSVC tools write object databases, tracker logs and try-compile scratch files through
MAX_PATH-limited interfaces. The audio supplier nests its own `ExternalProject` trees several
levels deep, so a build tree inside Cargo's hashed `OUT_DIR` pushes those files past 260
characters, and the nested configure reports the misleading `No CMAKE_C_COMPILER could be found`.

Windows supplier builds therefore run from `%LOCALAPPDATA%\knovum-build\<fingerprint>`, one tree
per `OUT_DIR`. Installed artifacts stay in `OUT_DIR`; `cargo clean` does not remove the build
trees, so delete that directory to reclaim space or to force a full supplier rebuild.

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

Browser-only runtime is not supported. Use `pnpm desktop:dev` so Tauri IPC, window APIs, system locale, capabilities, local credential database and lifecycle behavior are present.

## The database refuses to open

SQLx applies pending migrations before the repository pool becomes available, and refuses a database
it cannot explain.

A changed checksum means a migration that had already shipped was edited. Restore that file: the
shipped history is frozen, and the fix is a new numbered migration
([data-ownership.md](data-ownership.md) §Migration policy). A recorded version this binary does not
carry means the database was written by a newer release — install that release again; do not try to
make an older binary accept it.

Neither case is resolved by deleting a library. Never tell a user to remove `spaces.db`, and never
add a repair migration, compatibility reader, or parallel schema path.

For SQLite changes, `pnpm check:rust` exercises the embedded SQLx history against fresh in-memory
databases, and a new migration also carries a test that upgrades the previously shipped schema with
representative rows in it.
