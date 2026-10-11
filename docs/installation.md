# Installation

## Developer workstation

Knovum targets the Tauri desktop runtime.

Required:

- Node.js `>=22.13`;
- Corepack + pnpm `11.17.0` from `packageManager`;
- Rust toolchain from `rust-toolchain.toml`;
- uv `0.12.23`, CMake `4.3.4`, Protobuf `32.1`, and LLVM/libclang for the native worker;
- platform prerequisites required by Tauri for macOS or Windows.

First setup:

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm desktop:dev
```

The checkout includes the project lockfiles:

```text
pnpm-lock.yaml
src-tauri/Cargo.lock
python/uv.lock
```

Use the committed lockfiles for normal setup and CI. Regenerate a lockfile only for an intentional dependency change.

## Local validation

```bash
pnpm check
pnpm package --no-bundle --debug
```

`pnpm package --no-bundle --debug` compiles the desktop application without creating installers. The normal project builds supply all resources; no runtime publication is required. Packaging is documented in `packaging.md`.

## End-user installation

### macOS

The primary direct-distribution artifact is a signed and notarized `.dmg`. Users open the DMG and drag Knovum to Applications. Production builds distributed outside the Mac App Store must use the organization's Developer ID signing and notarization configuration.

### Windows

The primary interactive installer is the NSIS `Knovum-setup.exe`. MSI is an explicit stable-release artifact for managed/enterprise deployment because WiX/MSI does not accept prerelease SemVer strings such as `1.0.0-rc.x`. Production installers should be signed with the organization's Windows code-signing mechanism.

Do not document SmartScreen/Gatekeeper bypass instructions as the normal installation path. Fix signing/notarization instead.

## Application data

Application installation and application data are separate concerns. Do not delete user workspaces, databases, or credentials during an installer upgrade. Any future uninstall-data removal behavior must be an explicit product decision with migration/backup implications reviewed first.
