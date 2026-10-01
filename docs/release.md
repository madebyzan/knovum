# Production release

QA packaging and production publishing are different concerns. `packaging.md` owns local installer
creation and bundled-runtime provisioning. This document owns signed distribution, the stable update
channel and release acceptance. Workflow files implement the process; their existence does not prove
that signing credentials, environment protection or native upgrade acceptance have been configured.

## Release prerequisites

A production candidate must use committed pnpm/Cargo lockfiles and one reviewed commit on `main`.
The candidate workflow requires the tag to match all three application versions, reviewed notes at
`docs/release-notes/<version>.md`, dependency signature/advisory/license checks, `pnpm check`, native
builds, provisioned runtime smoke and the real background-removal smoke. The release matrix contains
macOS arm64/x64 and Windows arm64/x64. A partial matrix cannot enter the stable manifest.

Before promotion, also review new capability permissions, CSP origins, protocol/file handlers and
platform metadata. No WebView may receive SQL permissions or a generic SQL/runtime mutation command.
Exercise the full embedded Spaces SQLx migration history and packaged application upgrade path.
Fresh-database tests alone are not upgrade acceptance. Final product icons and publisher metadata
must be approved before the first public distribution.

FFmpeg/whisper.cpp archives must be provisioned for each target. Pinned yt-dlp/Deno downloads must
remain available and match their embedded hashes. Clean-profile tests must install that pair and
exercise representative PDF, media and transcription workflows. Hashes prove artifact identity,
not publisher signing or successful native execution. The inspected macOS arm64 yt-dlp standalone
asset is ad-hoc signed, not publisher-signed. Never remove quarantine metadata or weaken OS security
to make an unsigned tool run. Where policy requires publisher signatures, use controlled signed
artifacts, review their provenance/hashes and repeat native acceptance.

## Version and artifact identity

`src-tauri/tauri.conf.json` is the bundle version source. The packaging preflight requires its version
to equal `package.json` and the `[package]` version in `src-tauri/Cargo.toml`. Stable releases use plain
`major.minor.patch`; beta channels are not implicitly mixed into stable. Change all three version
fields in the release PR, review release notes, merge and tag that commit as `v<version>`.

A version is immutable. Never rebuild, re-sign or replace the artifacts associated with an existing
release/tag. An existing draft is not silently overwritten either. A failed candidate is investigated
and a new version is cut where its inputs must change. Hosting migration copies the already-approved
bytes and their original signatures; it does not create a new release identity.

## Permanent discovery, replaceable storage

The client embeds exactly one permanent HTTPS manifest URL and the public updater key. It consumes
the official Tauri static JSON format: `version`, `notes`, `pub_date`, and each platform's `url` and
`signature`. There is no custom discovery protocol, hard-coded mirror list, client cloud SDK, embedded
GitHub token, or disabled-signature escape hatch.

Keep private source and public distribution separate. Create an initialized **public distribution-only
repository** with default branch `main`; do not make `madebyzan/knovum` public. For example, after the
owner creates `madebyzan/knovum-releases`, the permanent endpoint can be:

```text
https://raw.githubusercontent.com/madebyzan/knovum-releases/main/channels/stable.json
```

That is a configuration example, not a provisioned service. GitHub Releases initially hosts the
signed installers/archives. Publishing writes `channels/stable.json` to the distribution repository
with a compare-and-swap on the previous file SHA. Private repository downloads cannot be used by
anonymous application clients; never solve that by putting a PAT into the desktop binary.

For independence from GitHub itself, select an **owner-controlled domain before the first public
release**. Serve the same channel file at its permanent path, initially through GitHub Pages or a
controlled edge, then change DNS/origin to COS/CDN later while preserving HTTPS and that path. The
publication workflow writes the canonical channel file in GitHub; an edge/custom-domain deployment
must serve that file before its post-publication check can pass. DNS, domain ownership and edge
configuration are infrastructure prerequisites, not fabricated by a repository build.

Old clients cannot discover a completely new endpoint if the only address they know is deleted.
Retain the original GitHub entry for the lifetime of those installations, or retain the same owned
domain. Merely embedding a different endpoint in the next binary strands dormant old clients.
Do not rotate the updater key as part of a hosting move.

### Moving artifacts to Tencent COS or another host

Copy the exact published version's archives/installers to permanent, anonymous HTTPS object URLs.
Do not use expiring signed query URLs, expose bucket write access, or change artifact bytes. Retain
the existing filenames. Run **Promote stable update** for that same version with `artifact_base_url`
set to the new version directory. The workflow verifies the candidate signatures, then downloads all
four public URLs without credentials and compares their SHA-256 hashes before changing discovery.
Same-version promotion may change URLs only, not signatures, notes or publication time.

Keep old download URLs working during migration so clients that already selected or started a
previous URL can finish. New checks, including checks from dormant installations, receive the new
locations from the unchanged manifest. Use short/revalidated caching for the channel JSON and
immutable long-lived caching for versioned artifacts. The final workflow check reads the permanent
endpoint and requires the promoted manifest. A stale edge may fail that check after a successful
channel commit; fix/purge its cache and repeat the same promotion rather than repacking the release.

## Signing decision and environment ownership

Updater signatures and operating-system publisher signatures are separate requirements.
`pnpm package:release` generates valid Tauri updater configuration before plugin initialization,
requires a signing key and builds Cargo with its lockfile. Ordinary QA packages remain updater-disabled
regardless of inherited updater environment variables. Missing production configuration fails the
build; it never produces a silently disabled production updater.

Create `release-signing` and `release-publish` GitHub environments. Configure their actual protection
rules: trusted `main`/release tags, required reviewers where the account plan supports them, restricted
administration and no untrusted-fork access. A YAML `environment` name alone does not create approval
protection. Promotion is separately manual even after candidate builds succeed.

Repository/environment variables, consistent between build and publication jobs:

| Variable                             | Value                                                                      |
| ------------------------------------ | -------------------------------------------------------------------------- |
| `KNOVUM_DISTRIBUTION_REPOSITORY`     | The public distribution-only `owner/repository`.                           |
| `KNOVUM_UPDATER_ENDPOINT`            | The permanent anonymous HTTPS channel JSON URL.                            |
| `KNOVUM_UPDATER_PUBKEY`              | Entire base64 public-key output produced by the Tauri signer, not a path.  |
| `APPLE_SIGNING_IDENTITY`             | Actual `Developer ID Application: ...` identity.                           |
| `APPLE_API_ISSUER`, `APPLE_API_KEY`  | App Store Connect notarization issuer and key IDs.                         |
| `KNOVUM_WINDOWS_SIGN_COMMAND`        | Explicit production signing command containing Tauri's `%1` file argument. |
| `AZURE_CLIENT_ID`, `AZURE_TENANT_ID` | Identity used by the selected Windows Artifact Signing client.             |

Secrets are scoped to the step/environment that needs them:

| Secret                                                            | Purpose                                                                            |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Updater signing; never shipped to clients.                                         |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`                 | Developer ID `.p12` material for the Tauri bundler.                                |
| `APPLE_API_PRIVATE_KEY`                                           | Notarization `.p8` contents; CI writes a restricted temporary file and removes it. |
| `AZURE_CLIENT_SECRET`                                             | Windows signing service credential.                                                |
| `KNOVUM_DISTRIBUTION_TOKEN`                                       | Fine-grained distribution-repository Contents write access only.                   |

The existing controlled-runtime URL/hash variables and optional runtime authorization secret remain
required as documented in `packaging.md`. Public verification keys are intentionally not secrets.
Generate updater keys with the official Tauri signer, outside the repository, and keep an encrypted
backup under release ownership. Loss of the private key breaks trust continuity for already-installed
clients. Key rotation requires a separately reviewed bridge/recovery plan; changing a CI variable is
not a rotation strategy.

macOS uses Developer ID signing and App Store Connect API notarization. The workflow verifies the app
with `codesign`, Gatekeeper assessment and stapled notarization validation, followed by clean-machine
installation/launch acceptance. Windows uses Tauri's explicit `signCommand` integration. The workflow
pins `artifact-signing-cli` 0.11.0; configure its endpoint/account/profile for the organization's
Microsoft Artifact Signing account, for example `artifact-signing-cli -e <service-endpoint> -a <account>
-c <profile> -d Knovum %1`. Account enrollment and credentials must exist before release. This is a
build-time mechanism, not an application dependency. Validate the application and NSIS installer's
Authenticode signatures and timestamps. Signing does not promise immediate SmartScreen reputation.

## Candidate, acceptance, promotion

`.github/workflows/release-candidate.yml` builds all four signed targets from a reviewed version tag.
It collects immutable uniquely named update artifacts and human-facing installers. `candidate.json`
records their hashes, original signatures, source commit, endpoint and public key. Collection rejects
mixed commits/versions/configurations. Publication verifies every updater artifact with the official
`minisign` verifier before creating a **draft** Release in the public distribution repository.

Perform native acceptance on those exact candidate bytes. Record the OS/architecture, source and
destination versions, source commit, hashes and observations at a reviewable evidence URL. Then run
`.github/workflows/promote-update.yml` from `main`, providing `version` and `upgrade_evidence_url`.
The evidence URL identifies reviewer-owned evidence; the workflow does not pretend that a URL proves
successful QA. Approvers must inspect it. An optional `artifact_base_url` selects COS/CDN storage.

Promotion re-verifies downloaded candidate signatures/hashes, refuses stable downgrades and immutable
version replacement, publishes the draft if needed, verifies anonymous artifact downloads, and only
then atomically updates the channel. A failure before that channel update leaves the current update
offer unchanged, even if the candidate release has become publicly downloadable. Concurrent writes
fail instead of silently overwriting another promotion. The final step checks the actual permanent
entrypoint; no successful build alone pushes an update to users.

## Client lifecycle contract

Rust owns the selected `Update`, verified bytes and single-flight operation lock. The frontend receives
strictly validated versioned snapshots; subscribing before the initial read and ignoring old revisions
prevents delayed messages from overwriting newer state. Settings is a view of one Pinia session, not
the owner of an installation. The direct native updater JS commands remain ungranted.

Enabled builds check after 30 seconds and every six hours, without interrupting startup or repeatedly
replacing a selected update. About also supports explicit checks. Downloads are user initiated,
cancellable and visible in Task Center/Dock/Taskbar. The official plugin verifies the signature before
the state becomes Ready. Files larger than 1 GiB are rejected by publishing and download policy;
Tauri buffers accepted archives in memory, so this is not a streaming-to-disk/resume implementation.
Exiting before installation discards the in-memory download and requires another download next time.

Only the version actually reviewed and downloaded may be installed. Installation is explicit, blocked
while other Task Center jobs run, and shares the existing registered-save barrier with ordinary quit.
All save attempts settle; a failed save prevents installation. A modal prevents new user edits while
preparing/installing, and task state is checked again after saving. Windows may exit from inside its
installer, so saving happens before invoking it. During native replacement, ordinary quit is blocked;
Tauri's distinguished restart exit is allowed. Network, signature and installation failures become
recoverable, visible update states, not a claim of success. Release notes are rendered as plain text.

## Required acceptance matrix

| Scenario                                            | Required outcome on every supported target                                    |
| --------------------------------------------------- | ----------------------------------------------------------------------------- |
| QA vs production startup                            | QA starts with updates disabled; correctly configured production initializes. |
| Installed N to N+1                                  | Check, download, verify, save, install and relaunch; new version is reported. |
| Data and migrations                                 | Existing spaces, assets, credentials, preferences and schema remain usable.   |
| Wrong key / tampered file                           | Installation is refused and the current app remains usable.                   |
| Offline / stalled service / invalid JSON            | Finite wait, visible failure, explicit retry; no silent success.              |
| Slow/unknown-size download and cancel               | Accurate/indeterminate progress, cancellation releases the operation.         |
| Repeated clicks / leave and reopen About            | One native download/install, retained progress and selected version.          |
| Save failure / active task / rejected OS permission | No unsafe install; failure is visible and the app remains usable.             |
| GitHub to COS migration                             | Original client finds the same signed version through unchanged discovery.    |
| Relaunch / native installer behavior                | No blocked restart, duplicate process or unexpected ordinary quit.            |

The first update-enabled public installation establishes the trust/endpoint baseline. A previously
shipped updater-disabled QA binary cannot be remotely taught an endpoint; install the baseline normally.
Test two real signed versions before advertising auto-update readiness. Unit tests, compile smoke and
crypto rejection tests complement this matrix, not replace it.

## Recovery and post-release

Monitor the real manifest and anonymous downloads after promotion, verify native signatures and
review sanitized update failures. Stop a bad rollout by removing that version from the channel or
restoring the previous known-good manifest through an explicitly reviewed emergency change. That
stops offers to old clients; it does not downgrade machines already upgraded.

Prefer a higher-version forward fix. Never automatically run an older binary against a newly migrated
database. After the first public persisted schema is frozen, maintain reviewed migration, backup and
recovery policy and test restored data against the intended binary. Keep rescue installers and the
previous release available. Do not claim automatic application/database rollback without evidence.
