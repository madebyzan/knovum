# Production release

Packaging and production publishing are different concerns. `docs/packaging.md` covers local/QA installer artifacts. This document is the production boundary.

## Release prerequisites

A production release is allowed only when:

- `pnpm-lock.yaml` and `src-tauri/Cargo.lock` are committed;
- `pnpm check` passes on the release commit;
- the Quality workflow passes its npm signature/advisory and Rust advisory/license gates on the release commit;
- macOS and Windows platform smoke builds pass;
- QA installation/upgrade smoke tests pass on supported OS versions;
- the application version and release notes are reviewed;
- all newly added capability permissions, CSP origins and protocol/file handlers are reviewed;
- `pnpm check:rust` exercises the complete embedded Spaces SQLx migration history against fresh
  SQLite databases, and the packaged application upgrade path has been exercised before release;
- capability review confirms that no WebView has SQL permissions and no generic Rust SQL/runtime-event mutation command is exposed;
- bundled FFmpeg/whisper.cpp archives for the release targets are provisioned;
- the pinned yt-dlp and Deno release assets remain available, match the embedded manifest hashes,
  and install successfully from a clean application-data directory;
- provisioned-runtime startup smoke starts FFmpeg and whisper.cpp `whisper-cli` with standalone VAD before bundling; clean-profile production feature smoke additionally installs the pinned yt-dlp/Deno pair and exercises representative PDF/media/transcription workflows on packaged artifacts;
- organization-owned signing/notarization credentials are configured in CI;
- updater configuration is complete if the release channel uses auto-update.

## Version source

`src-tauri/tauri.conf.json > version` is the application bundle version source. Keep package/Cargo metadata aligned when cutting a repository release so tooling, logs and artifacts report the same version.

Do not publish a mutable artifact under an existing version.

## On-demand media tool evidence

The embedded SHA-256 values prove that the downloaded yt-dlp and Deno files are the reviewed
artifacts; they do not prove publisher signing or successful execution on a supported operating
system. The currently inspected macOS arm64 Deno asset carries a Deno Land Developer ID signature,
while the inspected yt-dlp standalone asset is only ad-hoc signed. Do not describe that yt-dlp asset
as production publisher-signed.

Before public distribution, record clean-machine install and launch evidence for both tools on every
supported macOS and Windows target. If organization policy requires publisher-signed executables,
publish project-controlled signed artifacts, update the fixed manifest hashes and provenance, and
repeat those platform checks. The application must not remove quarantine metadata or weaken OS
security checks to make an unsigned artifact run.

## macOS production distribution

Direct distribution uses a DMG containing the app bundle.

Required before public distribution:

1. final application icon and bundle metadata;
2. Developer ID Application certificate;
3. code signing;
4. notarization;
5. stapling/verification;
6. install and first-launch smoke test on a clean machine/user profile.

CI secrets normally include the Apple certificate material and either App Store Connect API credentials or Apple ID notarization credentials. Do not commit certificate/private-key material.

## Windows production distribution

The default human-facing installer is NSIS `setup.exe`; MSI is retained for managed deployment requirements.

Required before public distribution:

1. final icon/publisher metadata;
2. organization-selected code-signing mechanism;
3. signed installer validation;
4. install/upgrade/uninstall smoke tests on supported Windows versions;
5. SmartScreen/reputation considerations reviewed for the selected certificate strategy.

The repository intentionally does not select PFX, Azure Key Vault, or Azure Artifact Signing on behalf of the organization. Choose the signing mechanism when credentials/operations ownership are known and record it in an ADR.

## Updater

The updater runtime and in-app About-page UX are installed. A release build becomes update-enabled only when CI embeds both values below:

```text
KNOVUM_UPDATER_ENDPOINT=https://updates.example.com/{{target}}/{{arch}}/{{current_version}}
KNOVUM_UPDATER_PUBKEY=<minisign public key>
```

Build production release artifacts with `pnpm package:release` (or the platform-specific `package:release:*` scripts). Those scripts merge `src-tauri/tauri.release.conf.json`, which enables `bundle.createUpdaterArtifacts`. Keep the private updater signing key only in the release environment and use Tauri's signing environment variables/process when packaging. The public verification key is intentionally build configuration, not a secret.

Before publishing an update-enabled channel:

- define release channels (for example stable/beta) and rollout policy;
- configure the real updater endpoint(s);
- create and protect the updater signing private key;
- embed the matching public verification key;
- generate and publish the signed updater artifacts/manifest;
- test Check → Download → Verify → Install → Restart from at least the previous supported production version;
- define rollback/recovery policy for a broken release.

Never disable update signature verification in production.

## GitHub release automation

Use the official Tauri `tauri-action` after organization signing is configured. The action can build desktop bundles and attach artifacts to a draft GitHub Release. Keep the release workflow draft-only until signing and updater secrets have been validated in the repository environment.

The base repository ships a safe manual QA packaging workflow instead of a fake production workflow with placeholder signing values. Add the production workflow in the same PR/ADR that chooses the organization's signing strategy.

## Release sequence

```text
release candidate
      |
quality + platform CI
      |
manual QA installer test
      |
version/release note review
      |
production signed/notarized packaging
      |
draft release validation
      |
publish release
      |
updater smoke check (when enabled)
      |
post-release monitoring
```

## Post-release

After publication:

- verify artifacts can be downloaded and installed from the real distribution channel;
- verify signature/notarization status;
- verify deep links/protocol handling if shipped;
- verify updater metadata when enabled;
- keep logs/diagnostics free of credentials;
- before the first public release, record any baseline/reset issue before beginning the next release; once a public persisted schema is frozen, define and verify the production migration/rollback policy explicitly.
