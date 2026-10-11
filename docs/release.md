# Production release

QA packaging and production publishing are different concerns. `packaging.md` owns local installer
creation and bundled-runtime provisioning. This document owns signed distribution, the stable update
channel and release acceptance. Workflow files implement the process; their existence does not prove
that signing credentials, environment protection or target-platform upgrade acceptance have been configured.

## Agent conversation pre-release contract cutover

The first-release contracts use the single Spaces v1 SQLx baseline, Space packages v1,
private Agent RPC v1, graph identity v1 and Builtin Agent profile v1. Ship matching Rust
and generated neutral contracts/worker source together. Old requests and packages fail strict
validation; old graph segments are not resumed through field conversion. An obsolete database
checksum produces a visible bootstrap error and preserves the original database. Backing up and
explicitly resetting pre-release data is a separate operation, never an automatic upgrade step.
The new release does not restore old Ask/Plan/Execute records by defaulting them to Execute.

## Release prerequisites

A production candidate must use committed pnpm/Cargo lockfiles and one reviewed commit on `main`.
The candidate workflow requires the tag to match all three application versions, reviewed notes at
`docs/release-notes/<version>.md`, dependency signature/advisory/license checks, `pnpm check`, target-platform
builds and provisioned runtime smoke. Real background-removal acceptance remains target-platform evidence. The release matrix contains
macOS arm64/x64 and Windows x64. A partial matrix cannot enter the stable manifest.
Unauthorized writes, lost edits, duplicate irreversible effects, post-cancel publication, false
success or evidence, secret leakage and unrecoverable data loss block release.
The independently published, on-demand CPython interpreter, compiled extension modules and dependent dynamic libraries are
part of the application trust boundary. Each target must verify the complete runtime manifest,
compiled-library loadability, signing/notarization or Authenticode conditions, and an installed inference
smoke on a clean machine without a system Python installation. A matching hash alone does not
establish code-signing trust or binary-wheel support.

Speech acceptance uses the exact candidate runtime and resources. On macOS arm64 it covers the MPS
Metal VoxCPM2 profile; Intel macOS and Windows also qualify the original-precision GGUF CPU profile with a minimum 32 GiB physical memory. These gates are not performance evidence: target-platform
QA must still record peak memory, latency, cancellation, recovery and output quality. Sherpa diarization
acceptance covers anonymous fixed-revision downloads, automatic and known speaker counts, and
offline inference without credentials or external configuration. Alignment acceptance covers the selected-range limits and manual timing invalidation.

### Target-platform inference quality evidence

Model qualification uses real candidate execution, behavioral tests and reviewed measurement
artifacts. The repository does not maintain a separate scoring CLI, corpus protocol or report
framework. Record application commit, worker/runtime digests, model snapshot and configuration,
input/output hashes, OS/architecture, device, driver and power mode with the measurements.
Missing execution or review evidence blocks qualification; synthetic unit fixtures and reference-only
runs cannot qualify the candidate. Retain raw results so a reviewer can reproduce the calculation.

Freeze the corpus, baseline and thresholds before measuring, and cover every supported target:

| Capability | Required evidence                                                                                                                                                                                                                                                                                                                   |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Embedding  | Documents, notes, code and transcripts with relevance judgments; official Sentence Transformers CPU/float32 reference; paired cosine at least 0.9999 and Recall@10/nDCG@10 loss at most 0.01.                                                                                                                                       |
| ASR        | Chinese/English human transcripts, mixed language, names, numbers, silence, noise, overlap and long recordings; at least two sample rates and one recording of at least ten minutes. Micro-aggregate CER/WER including insertions on silence; regression against the frozen matching model/configuration baseline above 0.01 fails. |
| Alignment  | Chinese/English human unit boundaries; coverage, median and P95 boundary errors against preregistered limits, by language and overall.                                                                                                                                                                                              |
| TTS        | At least 30 fixed Chinese/English statements, each reviewed by at least three independent blinded raters before assignment disclosure. Apply preregistered 1–5 mean-score limits for pronunciation, naturalness, voice consistency, reference similarity and emotion control. ASR round-trip scores do not replace listening.       |
| Images     | Model-specific output, visual review, memory, latency and active cancellation evidence described in `local-models.md`.                                                                                                                                                                                                              |

Licensed/private corpus data may stay in the controlled QA evidence store. Keep its version,
source/license, immutable input bytes and annotations reviewable. Measure actual load/warm latency,
peak memory, cancellation and recovery as well as output quality. Requalify changes to candidate
runtime, model, execution configuration or corpus; attach the record to the exact release artifacts.

The selected native Sherpa build disables TTS/eSpeak and contains only the required diarization
and separation closure. Its frozen supplier notices and source identity must be reviewed with the
candidate; the removed Python Sherpa wheel and its eSpeak closure are not distributed. Retained
Agent compiled dependencies still require their independent corresponding-source review.

Before promotion, also review new capability permissions, CSP origins, protocol/file handlers and
platform metadata. No WebView may receive SQL permissions or a generic SQL/runtime mutation command.
Exercise the full embedded Spaces SQLx migration history and packaged application upgrade path.
Fresh-database tests alone are not upgrade acceptance. Final product icons and publisher metadata
must be approved before the first public distribution.

FFmpeg, the sealed native inference package and the retained Agent runtime must be prepared for each target. Pinned yt-dlp/Deno downloads must
remain available and match their embedded hashes. Clean-profile tests must install that pair and
exercise representative PDF, media and transcription workflows. Hashes prove artifact identity,
not publisher signing or successful target execution. The inspected macOS arm64 yt-dlp standalone
asset is ad-hoc signed, not publisher-signed. Never remove quarantine metadata or weaken OS security
to make an unsigned tool run. Where policy requires publisher signatures, use controlled signed
artifacts, review their provenance/hashes and repeat target-platform acceptance.

Candidate metadata uses `localRuntimes`: one entry per target, containing the shipped
Worker tree digest and the one Agent descriptor plus native worker/runtime inventory digests. Collection and candidate
validation require the Agent profile exactly once and the native package inventory, matching the artifact target and the strict runtime
descriptor contract. Old `localInference` metadata cannot qualify a candidate.

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
two public URLs without credentials and compares their SHA-256 hashes before changing discovery.
Same-version promotion may change URLs only, not signatures, notes or publication time.

Keep old download URLs working during migration so clients that already selected or started a
previous URL can finish. New checks, including checks from dormant installations, receive the new
locations from the unchanged manifest. Use short/revalidated caching for the channel JSON and
immutable long-lived caching for versioned artifacts. The final workflow check reads the permanent
endpoint and requires the promoted manifest. A stale edge may fail that check after a successful
channel commit; fix/purge its cache and repeat the same promotion rather than repacking the release.

## Signing decision and environment ownership

Updater signatures and operating-system publisher signatures are separate requirements.
`pnpm package --release` generates valid Tauri updater configuration before plugin initialization,
requires a signing key and builds Cargo with its lockfile. Ordinary QA packages remain updater-disabled
regardless of inherited updater environment variables. Missing production configuration fails the
build; it never produces a silently disabled production updater.

Create `release-signing` and `release-publish` GitHub environments. Configure their actual protection
rules: trusted `main`/release tags, required reviewers where the account plan supports them, restricted
administration and no untrusted-fork access. A YAML `environment` name alone does not create approval
protection. Promotion is separately manual even after candidate builds succeed.

Repository/environment variables, consistent between build and publication jobs:

| Variable                             | Value                                                                         |
| ------------------------------------ | ----------------------------------------------------------------------------- |
| `KNOVUM_RUNTIME_PUBLICATION_TAG`     | Reviewed signed controlled-runtime release tag; required by candidate builds. |
| `KNOVUM_DISTRIBUTION_REPOSITORY`     | The public distribution-only `owner/repository`.                              |
| `KNOVUM_UPDATER_ENDPOINT`            | The permanent anonymous HTTPS channel JSON URL.                               |
| `KNOVUM_UPDATER_PUBKEY`              | Entire base64 public-key output produced by the Tauri signer, not a path.     |
| `APPLE_SIGNING_IDENTITY`             | Actual `Developer ID Application: ...` identity.                              |
| `APPLE_API_ISSUER`, `APPLE_API_KEY`  | App Store Connect notarization issuer and key IDs.                            |
| `KNOVUM_WINDOWS_SIGN_COMMAND`        | Explicit production signing command containing Tauri's `%1` file argument.    |
| `AZURE_CLIENT_ID`, `AZURE_TENANT_ID` | Identity used by the selected Windows Artifact Signing client.                |

Secrets are scoped to the step/environment that needs them:

| Secret                                                            | Purpose                                                                            |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Updater signing; never shipped to clients.                                         |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`                 | Developer ID `.p12` material for the Tauri bundler.                                |
| `APPLE_API_PRIVATE_KEY`                                           | Notarization `.p8` contents; CI writes a restricted temporary file and removes it. |
| `AZURE_CLIENT_SECRET`                                             | Windows signing service credential.                                                |
| `KNOVUM_DISTRIBUTION_TOKEN`                                       | Fine-grained distribution-repository Contents write access only.                   |

The selected controlled-runtime publication must be anonymously downloadable as documented in `packaging.md`; packaging obtains resource URLs and hashes from that manifest. Public verification keys are intentionally not secrets.
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

`.github/workflows/release-candidate.yml` builds all three supported signed targets from a reviewed version tag. First publish the controlled runtime with `python-runtime.yml` and `production: true`, then set `KNOVUM_RUNTIME_PUBLICATION_TAG` in `release-signing` to that immutable `runtime-*` release. The candidate downloads its publication manifest and validates current recipes, dependency identities, complete resources and production signatures; it reuses Agent/core-media publications and builds the native Rust worker and supplier closure from the candidate checkout.
It collects immutable uniquely named update artifacts and human-facing installers. `candidate.json`
records their hashes, original signatures, source commit, endpoint and public key. Collection rejects
mixed commits/versions/configurations. Publication verifies every updater artifact with the official
`minisign` verifier before creating a **draft** Release in the public distribution repository.

Perform target-platform acceptance on those exact candidate bytes. Record the OS/architecture, source and
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
the owner of an installation. The direct updater JS commands remain ungranted.

Enabled builds check after 30 seconds and every six hours, without interrupting startup or repeatedly
replacing a selected update. About also supports explicit checks. Downloads are user initiated,
cancellable and visible in Task Center/Dock/Taskbar. The official plugin verifies the signature before
the state becomes Ready. Files larger than 1 GiB are rejected by publishing and download policy;
Tauri buffers accepted archives in memory, so this is not a streaming-to-disk/resume implementation.
Cancellation and the transition to Ready are decided under the same updater state lock. A cancellation
accepted before completion discards the verified bytes; a request arriving after Ready is rejected
instead of falsely reporting that it cancelled an installable update.
Exiting before installation discards the in-memory download and requires another download next time.

Only the version actually reviewed and downloaded may be installed. Installation is explicit, blocked
while other Task Center jobs run, and shares the existing registered-save barrier with ordinary quit.
All save attempts settle; a failed save prevents installation. A shared lifecycle admission lock
excludes concurrent quit, hide-after-save and installation.
Application commands, including Application Menu commands and already-queued shortcut callbacks, cannot
start while that lock is held. The lock releases on save or installer failure. A modal prevents new
user edits while preparing/installing, and task state is checked again after saving. Windows may exit
from inside its installer, so saving happens before invoking it. During the system replacement transition, ordinary
quit is blocked; Tauri's distinguished restart exit is allowed. Network, signature and installation
failures become recoverable, visible update states, not a claim of success. Release notes are rendered
as plain text.

## Required acceptance matrix

| Scenario                                            | Required outcome on every supported target                                                                                        |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| QA vs production startup                            | QA starts with updates disabled; correctly configured production initializes.                                                     |
| Installed N to N+1                                  | Check, download, verify, save, install and relaunch; new version is reported.                                                     |
| Data and migrations                                 | Existing spaces, assets, credentials, preferences and schema remain usable.                                                       |
| Wrong key / tampered file                           | Installation is refused and the current app remains usable.                                                                       |
| Offline / stalled service / invalid JSON            | Finite wait, visible failure, explicit retry; no silent success.                                                                  |
| Slow/unknown-size download and cancel               | Accurate/indeterminate progress, cancellation releases the operation.                                                             |
| Repeated clicks / leave and reopen About            | One updater download/install, retained progress and selected version.                                                             |
| Save failure / active task / rejected OS permission | No unsafe install; failure is visible and the app remains usable.                                                                 |
| GitHub to COS migration                             | Original client finds the same signed version through unchanged discovery.                                                        |
| Relaunch / system installer behavior                | No blocked restart, duplicate process or unexpected ordinary quit.                                                                |
| Speech runtime and model resources                  | ASR, alignment, public diarization download, MOSS-TTS-Nano and compiled VoxCPM2 profiles are exercised on their supported target. |
| GPL source and notices                              | Candidate materials match every conveyed wheel/data hash and the approved distribution decision.                                  |

The first update-enabled public installation establishes the trust/endpoint baseline. A previously
shipped updater-disabled QA binary cannot be remotely taught an endpoint; install the baseline normally.
Test two real signed versions before advertising auto-update readiness. Unit tests, compile smoke and
crypto rejection tests complement this matrix, not replace it.

## Recovery and post-release

Monitor the real manifest and anonymous downloads after promotion, verify platform code signatures and
review sanitized update failures. Stop a bad rollout by removing that version from the channel or
restoring the previous known-good manifest through an explicitly reviewed emergency change. That
stops offers to old clients; it does not downgrade machines already upgraded.

Prefer a higher-version forward fix. Never automatically run an older binary against a newly migrated
database. After the first public persisted schema is frozen, maintain reviewed migration, backup and
recovery policy and test restored data against the intended binary. Keep rescue installers and the
previous release available. Do not claim automatic application/database rollback without evidence.

## Agent and native distribution acceptance

The desktop installer excludes CPython/third-party packages and includes the current application
Worker and trusted runtime descriptor. Target-platform acceptance must cover anonymous first download,
cancellation/retry, concurrent sharing and offline use after preparation, for Deep Agent. Native model execution uses the bundled Rust worker; native model weights are prepared before inference deadlines. Release
`platform.json`/`candidate.json` retain the Agent descriptor/tree digest and the native inference worker/manifest digests beside installer identities.
The Agent Worker digest hashes sorted `.py` entries as UTF-8 `path + NUL + file-sha256 + LF`,
with paths relative to the application-owned `python/src` tree. `scripts/runtime/packaged-resources.mjs` owns this
calculation. Agent execution binds that digest and the Agent descriptor's `manifestSha256`.
Inference capture provenance and every candidate execution source bind the bundled Rust binary
digest and the sealed native runtime manifest digest; outputs from another worker or runtime are rejected.
Runtime archives and corresponding evidence/source attachments must be published at the actual
immutable URLs before distributing an application that requires them. Workflow artifact uploads
are build outputs, not confirmation that those public locations exist.

Resource acceptance requires each descriptor's single `archiveUrl` to serve the declared SHA-256
without credentials, and every catalog-selected Hub source to pass application download/cancel/resume
and offline inference acceptance on supported networks. ModelScope content identity evidence does
not establish full transfer or target execution. Clients do not ask users for regions or infer them;
CDN routing belongs to the publisher. Current upstream URLs have not been qualified for every
mainland network. If a route fails acceptance, publish a permitted identical qualified copy and
update the catalog before release. Legacy dual-region descriptors are rejected.
MOSS-TTS-Nano and compiled VoxCPM2 require their exact source/model notices and platform qualification. Public Sherpa diarization requires target-platform acceptance
for anonymous download, cancellation, digest verification and subsequent offline inference.
Automatic speaker counting and turn quality are separate from known-count sample correctness.
No manual folder import or application-owned terms declaration remains.
No provider failover, compatibility endpoint precedence or replacement model satisfies this gate.

## Deep Agents acceptance

The Agent Runtime is an independent profile with its own descriptor, installed inventory and
license policy. The current locked SDK proof suite uses actual public Deep Agents/LangGraph APIs,
real SQLite persistence and controlled model responses/HTTP streams. Those checks establish
host admission, durable tool intent, projection and replay behavior; they do not qualify a real
provider, a model's quality, system TLS/proxy behavior or either release platform.

Run `pnpm contracts:generate`, `pnpm check:web`, `pnpm check:rust`, `pnpm check:python` on the same
checkout before target-platform acceptance. `check:python` shares Ruff policy but runs Builtin Agent Mypy/pytest
inside its own locked environment. Provision/build commands select the retained Agent explicitly.
Packaging verifies its descriptor and ships only current owned source. The maintainer owns target build/signing/installation acceptance.

### Provider qualification record

For each enabled model, record provider, immutable model identifier, base URL, explicit protocol,
context/output limits, temperature, OS/architecture, application commit, SDK lock SHA-256,
Agent Runtime manifest digest, date and input/result evidence. A passing OpenAI mock is not evidence
for Responses, Anthropic, Gemini, Ollama, DeepSeek, Groq or a compatible service.

| Endpoint family         | Required evidence                                                             | Current real-provider qualification |
| ----------------------- | ----------------------------------------------------------------------------- | ----------------------------------- |
| OpenAI Chat             | streaming usage, simultaneous tools/images, interruption/recovery             | Pending credentials/target run      |
| OpenAI Responses        | reasoning/tool correlation, encrypted reasoning, no response-store dependency | Pending credentials/target run      |
| Anthropic               | provider tool/image blocks and usage under summary/research                   | Pending credentials/target run      |
| Gemini v1/v1beta        | declared API version, tool/image content and usage                            | Pending credentials/target run      |
| Ollama                  | selected local model supports tools/images and reports usage                  | Pending configured model/target run |
| DeepSeek / Groq         | provider integration preserves reasoning/tool sequence and usage              | Pending credentials/target run      |
| China/custom compatible | exact declared endpoint/protocol, regional access, tool/image/usage           | Pending per-endpoint run            |

### Business regression corpus

Prepare one disposable Space with a note containing identifiable facts, a second contradictory
note, a PNG with text and shapes, a multipage document, timed media with transcript and configured
generation/export routes. Record exact source revisions and prompts before either engine is run.
Use the last pre-migration commit as the baseline and the candidate checkout as the new engine;
no old harness is installed into the candidate. Run each case at least three times with the same
model, sampler, input and Action catalog. Store input, public messages, Run events, Action receipts,
request purpose/count, token totals and elapsed time per run. This comparison has not been
performed with real credentials; qualification remains pending.

| #   | Task/input                                       | Success criterion                                                                  |
| --- | ------------------------------------------------ | ---------------------------------------------------------------------------------- |
| 1   | Ask for the attached note's three facts          | Correct facts and node/revision citations; no writes                               |
| 2   | Search for an exact phrase in another note       | Authorized matching node found                                                     |
| 3   | Compare contradictory notes                      | Both cited; discrepancy explained                                                  |
| 4   | Ask about an unindexed canvas item               | Reads the existing node, without invented contents                                 |
| 5   | Request inaccessible Space data                  | Grant denies access; no leaked content                                             |
| 6   | Ask to publish a note in Ask mode                | No write Action advertised or executed                                             |
| 7   | Plan a note rewrite                              | Visible pending/completed todos; no mutation                                       |
| 8   | Execute an authorized note rewrite               | One revisioned Action receipt and result                                           |
| 9   | Execute creation of two notes                    | Two requested outputs, each exactly once                                           |
| 10  | Ask about objects in attached PNG                | Actual authorized image input; correct visual evidence                             |
| 11  | Read a canvas image through a tool               | Rendered source revision supplied to model                                         |
| 12  | Send image to a text-only model                  | Rejected before model request                                                      |
| 13  | Read a multipage document excerpt                | Exact source locations cited                                                       |
| 14  | Find a phrase in a timed media transcript        | Source node, revision and timestamp match                                          |
| 15  | Generate an image                                | Uses configured image route; one owned child Run                                   |
| 16  | Generate a video                                 | Uses configured video route; no repeated child submission                          |
| 17  | Export a document                                | Valid published asset and Action receipt                                           |
| 18  | Generate speech with selected voice              | Existing speech Action result and immutable voice revision                         |
| 19  | Reuse a published output in a follow-up          | Reads current node rather than a scratch-path claim                                |
| 20  | Follow up using same model/mode                  | Same successful execution segment; one new input                                   |
| 21  | Switch model/protocol profile                    | New frozen execution segment; public history retained                              |
| 22  | Branch at an earlier message                     | New thread sees only chosen branch history                                         |
| 23  | Trigger context compression                      | Older complete answers remain public; summary is private                           |
| 24  | Delegate two independent research questions      | At most two read-only instances; no nested delegation                              |
| 25  | Attempt write/research in same tool batch        | Business write denied; no unsafe concurrent change                                 |
| 26  | Pause during a stream, then resume               | Safe boundary acknowledged; input not duplicated                                   |
| 27  | Pause while waiting for generation               | Resume observes the same child Run                                                 |
| 28  | Cancel during generation                         | Root/owned child fence blocks late publication                                     |
| 29  | Kill process after a submitted tool and reopen   | Exactly one Action result and public answer                                        |
| 30  | Lose provider response / usage or projection ACK | Correct distinct waiting reason or unique rebuilt answer; no automatic paid resend |

Zero authorization violations or duplicate business effects are required. Investigate any core
feature loss. More than 20% median cost/time increase requires evidence, a strategy correction
and remeasurement; do not declare improvement from a constructed mock.

### Managed-runtime/control matrix

On **macOS 14+ arm64/x64** and **Windows x64**, record clean installation without system Python,
anonymous first download of the retained Agent through the application-managed sources, cancellation, dependency
imports, restart/offline use, system CA/proxy behavior, protocol initialization, process-tree
termination, application exit, checkpoint/input recovery, Task Center waiting/pause/cancel and
asset publication. The producer performs relocation imports from a path with spaces and Chinese
characters before and after signing. Verify compiled binary signers/timestamps, app signing,
install/uninstall and upgrade separately. macOS x64 development checks do not satisfy this matrix.

## Agent extension qualification

Code verification covers package component isolation/path boundaries, immutable revisions,
SDK MCP catalog/invocation, OAuth credential lifecycle, durable approval/unknown attempts,
selected Skill bytes and ACP task isolation. Real provider OAuth registration, actual service
side effects and both target operating systems require separately recorded release evidence.
Do not treat SDK fixtures as certification of every third-party service. Include the bundled
standard plugin in resource/signature inventories and dependency SBOM/license review. Measure
the release binary/package delta for gix and rmcp client/auth features on each target; no package
size result is inferred from source or a development binary. Signing, installer acceptance and
physical platform qualification are maintainer release tasks.

## Native inference candidate identity

The native package is built and sealed before host compilation. Record the SHA-256 of the shipped
Rust worker, final signed native manifest and complete compiled-library inventory alongside each
target artifact. Inference capture uses these native digests; the Python tree digest belongs only to
Deep Agent and cannot authorize native inference evidence. The Node acceptance CLI keeps strict
hash/provenance checks, CER/WER micro-aggregation, retrieval, alignment and independent blind TTS
review thresholds. Synthetic tests prove the evaluator, never model or hardware qualification.

Real model smoke, F32 reference equivalence and stream/backpressure tests supplement the fixed
quality corpus. They do not replace visual review, bilingual ASR/alignment quality or thirty-sample
blind voice review. A platform unavailable to the implementer remains explicitly unqualified until
its actual CI/package/hardware acceptance completes. No unmeasured target is reported as passed.
