# Installation

## Developer workstation

Knovum targets the Tauri desktop runtime.

Required:

- Node.js `>=22.12`;
- Corepack + pnpm `11.17.0` from `packageManager`;
- Rust toolchain from `rust-toolchain.toml`;
- platform prerequisites required by Tauri for macOS or Windows.

First setup:

```bash
corepack enable
corepack prepare pnpm@11.17.0 --activate
pnpm install
cargo generate-lockfile --manifest-path src-tauri/Cargo.toml
pnpm desktop:dev
```

On macOS, install a stable Apple Development code-signing identity in the login Keychain and set `KNOVUM_DEV_SIGNING_IDENTITY` to its certificate name or SHA-1 hash before `pnpm desktop:dev`. Find available identities with `security find-identity -v -p codesigning`. The development launcher signs each rebuilt executable before Cargo runs it; it refuses an absent or ad-hoc (`-`) identity. This keeps macOS Keychain authorization tied to the same application identity across rebuilds. The first access to the vault may still require one OS approval.

Commit both lockfiles after the first successful resolution:

```text
pnpm-lock.yaml
src-tauri/Cargo.lock
```

Use `pnpm install --frozen-lockfile` in CI and normal reproducible setup after the lockfile exists.

## Local validation

```bash
pnpm check
pnpm desktop:build
```

`desktop:build` compiles the desktop application without creating installers. Packaging is documented in `packaging.md`.

## End-user installation

### macOS

The primary direct-distribution artifact is a signed and notarized `.dmg`. Users open the DMG and drag Knovum to Applications. Production builds distributed outside the Mac App Store must use the organization's Developer ID signing and notarization configuration.

### Windows

The primary interactive installer is the NSIS `Knovum-setup.exe`. MSI is an explicit stable-release artifact for managed/enterprise deployment because WiX/MSI does not accept prerelease SemVer strings such as `1.0.0-rc.x`. Production installers should be signed with the organization's Windows code-signing mechanism.

Do not document SmartScreen/Gatekeeper bypass instructions as the normal installation path. Fix signing/notarization instead.

## Application data

Application installation and application data are separate concerns. Do not delete user workspaces, databases, or credentials during an installer upgrade. Any future uninstall-data removal behavior must be an explicit product decision with migration/backup implications reviewed first.
