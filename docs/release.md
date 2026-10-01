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

The release workflow imports an organization-provided PFX into the ephemeral runner's current-user
certificate store; Tauri signs with that certificate's thumbprint and a configured HTTPS RFC 3161
timestamp service. This requires a legitimately issued certificate with an exportable key. It is not
a self-signed-certificate distribution strategy. Hardware-backed or cloud-only keys require replacing
the workflow's certificate provisioning with the organization's signer integration, not changing the
client update protocol or weakening signature validation.

## Updater configuration

`tools/package-desktop.mjs --release` is the update-enabled packaging entrypoint. It validates
`KNOVUM_UPDATER_ENDPOINT`, `KNOVUM_UPDATER_PUBKEY`, and `TAURI_SIGNING_PRIVATE_KEY` before starting
Tauri. Inherited `TAURI_CONFIG` overrides and release-bypassing CLI flags are rejected. The endpoint must be one stable, absolute HTTPS URL for a static Tauri update manifest,
without credentials, query tokens, fragments, or template variables. The public key must be the
base64-encoded public-key file generated by Tauri's signer; it is not a path or a secret.

The packaging process merges the public endpoint/key into `plugins.updater` before Rust compilation.
`tauri.release.conf.json` owns `bundle.createUpdaterArtifacts`; it must not contain an empty updater
configuration. Rust uses the embedded Tauri configuration as its single source of truth, validates
it at startup, and registers the official updater plugin only in a non-debug, update-enabled build.
Ordinary QA packages do not enable the updater merely because signing environment variables exist.
A malformed enabled configuration is an error, not a request to silently disable updating.

Keep `TAURI_SIGNING_PRIVATE_KEY` and its optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in the
protected signing environment. Only public configuration is passed in the Tauri configuration
argument. Do not place a private key, GitHub token, COS credential, or expiring download URL in the
application or update manifest. Windows uses the plugin's passive installer mode. Discovery has a
20-second request timeout; an archive download has a separate 30-minute budget. A timeout is surfaced
as a failure, not retried by a second provider behind the user's back.

## Client update lifecycle

Rust owns one update session and one non-queuing operation lock. All check, download, and install
commands share that lock. The check retains the official Tauri `Update` object; later commands must
supply its displayed version, and never query the server again during download or installation.
The updater plugin remains inaccessible directly from the WebView. The main-window capability and
Tauri application command manifest authorize only the five Knovum update commands.

The native scheduler checks once after 60 seconds, then every six hours while no candidate or
operation is pending. It does not block offline startup, catch up missed checks in bursts, download
without consent, or install silently. Manual checks are always available when idle. This initial
channel is stable only; prereleases and downgrades are not accepted.

The session progresses through Checking, Available, Downloading, Verifying, Ready, and Installing.
Downloads use the official plugin's signature verifier. Only its successful return can place bytes
in Ready. Download cancellation and Ready publication share the session lock, so an accepted
cancellation cannot race into an installable candidate. Cancelling or failing retains the candidate
for an explicit retry; a fresh check discards it. The client follows HTTPS-only redirects and uses
connection/read/total budgets. Signature failures do not authorize unsigned manual installation.

A Tauri Channel delivers revisioned snapshots to a single Pinia projection. Settings remounts do not
restart operations; reattaching receives an atomic snapshot. A detached WebView does not cancel a
native download. Task Center mirrors download progress/cancellation, and release notes render as
plain text. Candidate/download bytes are process-local, not a second persistent installer cache;
quitting before installation means a subsequent session checks and downloads again.

Restart/install is explicit. It waits for the same registered save callbacks used by ordinary
close/quit, waits for all callbacks even if one fails, and refuses installation if saving fails or
Task Center still has running work. A modal and the application command dispatcher block user actions
during this boundary. Windows exits inside the native installer; macOS restarts only after success,
with the desktop exit coordinator approving the restart rather than replacing it with exit(0).
Installation failure keeps the verified package available for an explicit retry when possible.
This is coordination of registered saves and presented tasks, not a claim that arbitrary external
processes or every future background job are automatically made safe; new work must participate in
the existing save/task contracts and the upgrade acceptance tests below.

Never disable update signature verification in production.

## Stable discovery and replaceable artifact storage

The client knows one stable discovery URL. The Tauri manifest at that URL points to versioned
artifacts; it is not tied to the storage provider containing those artifacts. Initially the artifact
URLs can use a public GitHub Releases repository. Later manifests can point to Tencent COS or
another HTTPS store without a client code change. Do not build a provider-selection SDK, a custom
update protocol, a second configuration-discovery service, or a compatibility fallback chain.

For provider-independent discovery, choose a project-controlled hostname and path before the first
public build. Keep that hostname, TLS service, and path available while changing its origin. A
custom hostname must terminate HTTPS through a configured static host/CDN/proxy; simply pointing a
CNAME at `github.com` does not configure GitHub Releases to serve that hostname.

A GitHub-native initial discovery URL may use this documented shape:

```text
https://github.com/OWNER/PUBLIC_DISTRIBUTION_REPOSITORY/releases/latest/download/latest.json
```

This is a format, not a provisioned Knovum endpoint. The private source repository must remain
private. Select a separate public distribution repository or a publicly readable COS distribution
path; never ship repository credentials to end users. Draft releases are not a public updater feed.

A GitHub-native discovery URL can survive a move of artifact bytes to COS as long as the old
GitHub discovery feed remains available and publishes manifests pointing to COS. It cannot make a
long-offline client discover a different endpoint after the original endpoint has been deleted.
Shipping one bridge version is not sufficient for clients that never installed that version. To
leave GitHub entirely without that dependency, use an owned discovery hostname from the outset.

The signing trust anchor is independent of storage. Copy already-signed bytes without rebuilding,
modifying, or re-signing them for a hosting move. Preserve the client-trusted key. Key loss or rotation
requires a separately designed trust migration; changing a URL alone cannot repair it.

## Preparing a GitHub release draft

`.github/workflows/release.yml` builds a signed release candidate from an explicitly selected
`vMAJOR.MINOR.PATCH` tag. It checks matching package/Cargo/Tauri versions, committed lockfiles and
reviewed `releases/VERSION.md`, runs quality/security gates and contract generation, provisions the
bundled runtime, builds all four native targets, validates platform signing, stages immutable names,
and verifies every updater signature before creating a public-repository **draft**. No job promotes
a release to users automatically. Protected environment approval and real upgrade QA are required.

The release tooling uses Node built-ins, the maintained `minisign` verifier, and GitHub CLI. Tauri
still owns packaging, signing and installation. No hosting SDK or custom cryptographic verifier is
introduced. `package.yml` remains the separate ad-hoc QA workflow; `updater-release-policy.yml`
executes lightweight orchestration/save-barrier tests on Linux and Windows without signing secrets.

The stable release matrix matches the four targets understood by the packaging entrypoint:
`darwin-aarch64`, `darwin-x86_64`, `windows-aarch64`, and `windows-x86_64`. All four are required by
this tooling; target names are not evidence that native QA has passed. A release owner must confirm
this support policy and run the target-specific checks before a public release. Stable versions
use plain `MAJOR.MINOR.PATCH`; beta channels are outside this change.

After provisioning bundled runtimes and completing platform signing/notarization, build each target
through `pnpm package:release:macos` or `pnpm package:release:windows` with its explicit `--target`.
Stage the updater artifact and adjacent `.sig` file from the corresponding Tauri bundle directory:

```sh
node tools/release/stage.mjs VERSION TAURI_OS_ARCH BUNDLE_DIRECTORY STAGING_DIRECTORY
```

The staging tool gives each artifact an unambiguous version/architecture filename. It rejects
missing or multiple candidates and refuses to overwrite a staged file. Use a clean staging
directory for a failed staging attempt; do not treat partially staged output as a release.

On the release host, install `gh` and `minisign`, authenticate `gh` with narrowly scoped access to
the chosen distribution repository, and set `KNOVUM_RELEASE_REPOSITORY` and
`KNOVUM_UPDATER_PUBKEY`. Then prepare a draft from the collected artifacts and reviewed release notes:

```sh
node tools/release/prepare-draft.mjs VERSION STAGING_DIRECTORY RELEASE_NOTES_FILE
```

The command requires all targets, checks every signature with `minisign`, confirms that the
distribution repository is public, creates a draft, and uploads the artifacts, signatures, and
`latest.json`. It never publishes, overwrites an existing release, or replaces a release asset.
Verification/visibility failures stop before release writes. An upload failure leaves a draft for
explicit inspection; it is not reported as success. The script consumes outputs of the tag-pinned native build; using it independently does not prove
source provenance or platform signing. Those checks remain mandatory before promotion.

Do not publish the resulting draft to a live channel until native signing, source/version
provenance, public download verification, and old-client upgrade tests have passed. GitHub CLI
credentials are release-host credentials only. The draft is for review, not a public update feed. Retain the build run URL, source SHA, version,
signer identity and native upgrade evidence with the release review.

## Moving artifacts to COS

Upload the identical immutable artifacts under their versioned names to the new HTTPS origin.
Verify their hashes and signatures and test actual downloads from the supported client platforms.
Generate a relocated manifest without overwriting the source:

```sh
node tools/release/relocate.mjs CURRENT_MANIFEST HTTPS_ARTIFACT_BASE NEW_MANIFEST
```

Verify the relocated, deployed bytes before promotion:

```sh
node tools/release/verify-hosted.mjs NEW_MANIFEST
```

`verify-hosted` uses curl with HTTPS-only redirects and minisign with the release public key. A
failed download or signature aborts verification and removes temporary files. It does not publish
anything or claim that the native install will succeed.

The offline relocation command preserves the version, notes, publication date, and signatures, changing only
the artifact URL base. It does not upload files, configure COS/DNS/TLS, or verify that the new URLs
are reachable. Validate the uploaded bytes before activating the resulting manifest at the stable
client-known entrypoint. Upload artifacts first and promote the manifest last. Give the mutable
manifest a short/revalidated cache lifetime; versioned bytes can be immutable. Keep the previous
origin available through the cache/in-flight-request transition.

Do not overwrite an asset in an immutable GitHub release. An owned discovery feed can promote a
new manifest independently; a GitHub release feed must publish a new reviewed release rather than
silently modifying an immutable published release. Keep a documented stop-push/recovery procedure;
do not solve a broken release by disabling signature or version checks or assuming that a database
migration can be reversed by downgrading the executable.

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

## One-time release environment setup

Create a **separate public distribution repository** with an initial README commit, leaving this source repository private. Choose
its permanent discovery URL (preferably an owned hostname) before distributing the first enabled
build. Do not invent a default hostname or embed a GitHub access token in the application.

Configure protected GitHub Environments named `release-validation` and `release`.
The workflow defaults to `release-validation`; give it a separate test feed/repository and updater
key. Production selects `release` explicitly. Restrict it to reviewed version tags and
require approval. Protect the main branch/tags, keep the workflow token read-only, and give the draft
publishing token Contents:write access to the distribution repository only. Public values are GitHub
variables; private material is GitHub Environment secrets:

| Kind     | Name                                                | Value                                             |
| -------- | --------------------------------------------------- | ------------------------------------------------- |
| Variable | `KNOVUM_RELEASE_REPOSITORY`                         | Actual public `owner/repository`                  |
| Variable | `KNOVUM_UPDATER_ENDPOINT`                           | Permanent public HTTPS static manifest URL        |
| Variable | `KNOVUM_UPDATER_PUBKEY`                             | Entire base64 public-key file from Tauri signer   |
| Secret   | `TAURI_SIGNING_PRIVATE_KEY`                         | Entire private-key file, not the public key       |
| Secret   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`                | Updater key password                              |
| Secret   | `KNOVUM_RELEASE_TOKEN`                              | Fine-grained distribution-repository token        |
| Secret   | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`   | Developer ID P12 in base64 and export password    |
| Variable | `APPLE_SIGNING_IDENTITY`                            | Full `Developer ID Application: ...` identity     |
| Variable | `APPLE_API_KEY`, `APPLE_API_ISSUER`                 | App Store Connect key ID and issuer               |
| Secret   | `APPLE_API_PRIVATE_KEY`                             | Contents of the downloaded P8 key                 |
| Secret   | `KNOVUM_WINDOWS_PFX`, `KNOVUM_WINDOWS_PFX_PASSWORD` | Issued Windows signing PFX in base64 and password |
| Variable | `KNOVUM_WINDOWS_TIMESTAMP_URL`                      | Signer's HTTPS RFC 3161 timestamp service         |

Also configure the existing `KNOVUM_BUNDLED_RUNTIME_*_URL`/`*_SHA256` variables and optional runtime
authorization secret documented in `packaging.md` for all four targets. Do not give release signing
secrets to pull-request jobs. The workflow removes temporary Apple API keys and Windows private keys;
GitHub-hosted signing runners must remain ephemeral.

Generate the updater key on the release owner's machine, outside the repository:

```sh
mkdir -p "$HOME/.tauri"
pnpm exec tauri signer generate -w "$HOME/.tauri/knovum.key"
```

Store the private key/password in an organizational secret manager and offline recovery backup.
Upload its secret through the GitHub UI/CLI without putting it into shell history or chat. Never
regenerate a production key just because storage changes. The key files must not be committed.

## Acceptance procedure before first promotion

Use the full checkout with the project's Node/pnpm and Rust toolchains. These are required checks,
not evidence that an assistant or a release-tool unit test has run them:

```sh
pnpm install --frozen-lockfile
node --experimental-strip-types --test tools/release/*.test.mjs tools/update-tests/*.test.mjs
pnpm contracts:generate
git diff --exit-code -- src/shared/contracts/generated
pnpm check
pnpm security:web
```

The Rust tests in `src-tauri/src/updater/tests.rs` cover configuration, candidate/version policy,
operation exclusion, cancellation/Ready races and byte retention after failures. The Vitest tests in
`src/foundation/updates/__tests__` exercise session persistence, save barriers, tasks, duplicate
requests and malformed/stale events. Node tests mock external tools deliberately; they are **not**
cryptographic or native-install tests. Regenerate contracts and run Rust/Oxc gates on the full repo;
do not bypass failures or mistake syntax-only checks for typechecking.

For a candidate, align the three version fields, add reviewed `releases/VERSION.md`, commit and tag
that commit. After the workflow is present on main, run it on the reviewed tag:

```sh
gh workflow run release.yml --repo madebyzan/knovum --ref vVERSION -f environment=release-validation
gh run list --repo madebyzan/knovum --workflow release.yml
```

A completed draft is still not release acceptance. Use an isolated HTTPS test feed with a separate
test signing key to build and install version N and then N+1 on clean macOS/Windows machines of
every advertised architecture. Keep the bundle identifier/install path stable within that test.
Do not put a production client on a test feed, inject testing bypasses into production code, or use
`tauri dev` as proof of installed-release behavior. Draft assets require authentication and cannot
serve as an anonymous test feed; use an isolated public test repository or owned test host.

| Scenario                                                          | Required observable result                                                                                   |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| QA/disabled build and enabled release cold launch                 | Both launch; only the enabled build checks after 60 seconds; offline startup remains usable                  |
| N to N+1 via actual feed                                          | Displayed version, downloaded candidate and installed version agree; restart opens N+1                       |
| Candidate changes on the server after check                       | The selected candidate is retained; a new candidate requires an explicit new check                           |
| Leave/reopen Settings during download                             | Same native task and monotonic state, no duplicate download                                                  |
| Cancel download, including at completion                          | No installable bytes after accepted cancellation; an explicit retry works                                    |
| Invalid manifest/platform, offline, timeout, interrupted download | Visible recoverable error; existing installation remains usable                                              |
| Wrong key or a flipped byte in the package                        | Verification fails; never reaches Ready; no installer is launched                                            |
| Running transcription/import/AI task                              | Install waits for the user to finish/cancel work; no forced termination                                      |
| Unsaved edits and delayed/failed save                             | All registered saves settle; failure blocks installation; ordinary quit/close still works afterward          |
| Native menu/shortcut during installation preparation              | No new application command starts; modal cannot be dismissed into editing                                    |
| Disk full, permission denial, cancelled native authorization      | No reported success; retry or documented manual recovery remains available                                   |
| Existing spaces, assets, settings and SQLite migrations           | Data remains readable and editable after restart; validate realistic old databases, not only a fresh profile |
| GitHub to COS with an old installed client                        | Without rebuilding that client, it reads the same feed and verifies/installs bytes from the new store        |
| Cache/long-offline clients                                        | Previous discovery URL still works; old cached/in-flight payload URLs remain available during migration      |

On macOS record `codesign --verify --deep --strict`, `spctl --assess --type execute`, and stapler
validation on the actual distributed bundle. On Windows record `Get-AuthenticodeSignature` and
clean-user install/upgrade/uninstall. Keep screenshots/logs, source commit, target, N/N+1 versions,
feed URL, signer identity and pass/fail results. Do not delete the application's data directory to
make an upgrade test pass. First-time install, OS reputation and migration checks remain separate
from the cryptographic updater check.

After validation tests, build the reviewed production tag with `-f environment=release`, validate
those production artifacts, and publish the reviewed production draft explicitly (GitHub Release UI, or `gh release edit vVERSION
--repo OWNER/DISTRIBUTION --draft=false`). For a GitHub latest-release discovery URL this makes the
feed live immediately. With an owned discovery URL, first verify public downloads with
`verify-hosted`, then atomically promote `latest.json` at that URL. Never announce readiness before
an anonymously reachable feed and all four valid artifacts are present.

## Recovery policy

Pause promotion when error reports appear. A controllable discovery feed may stop offering updates
(HTTP 204) while investigating; do not delete the permanent endpoint. Preserve the last known-good
installers and a manual download path. Prefer a tested **higher-version roll-forward** restoring good
behavior. Do not lower version comparisons, disable signatures, wipe AppData or assume executable
downgrade reverses a SQLite migration. Already-installed broken versions may require a manual signed
installer; already-downloaded candidates are intentionally pinned until users check again. Database
backup/restore and migration recovery must be tested against the actual persisted schema before a
public schema-changing release. Endpoint ownership and signing-key recovery are operational
requirements, not guarantees that application code can create by itself.
