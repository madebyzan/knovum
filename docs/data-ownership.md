# Local data ownership

SQLite is a storage primitive, not a shared application service.

## Rule

Spaces owns its persisted domain data, schema, Rust repository and tests. Do not create a global `DatabaseService` or expose arbitrary SQL across IPC.

The application database is `spaces.db`. SQLx owns its current v1 baseline in
`src-tauri/migrations/spaces/` and records the installed version and checksum in `_sqlx_migrations`.

The database resolves from Tauri's AppConfig directory through the connection policy in `src-tauri/src/sqlite.rs`. Spaces repositories own their SQL and domain rules.

## Migration policy

Before the first public release, Knovum-owned data formats, packages, private protocols, runtime
manifests/descriptors and the Agent profile/graph identity start at version 1 with their current
fields. Dependency and external API versions remain upstream-owned. Record revisions, Run state
versions and saved Workflow definition versions still advance normally. Renumbering does not
convert development data or runtime publications; unsupported formats fail strict validation.

Before the first public release, Spaces owns one SQLx v1 baseline. Schema changes update that
baseline and require deleting obsolete development databases. The embedded SQLx migrator runs
before the Spaces pool is exposed, installs the baseline transactionally, and rejects checksum
divergence. Store, package and protocol envelopes remain separate contracts with their own explicit versions.
Spaces conversations contain no product mode. The current Space package is v1 and the private
Agent RPC is v1; removing conversation modes changes both strict contracts without a compatibility
reader. Obsolete development databases fail visibly and are never reset automatically.

Install the matching SQLx CLI with
`cargo install sqlx-cli --no-default-features --features sqlite --version 0.8.6 --locked`.
`pnpm check:rust` runs Rust tests against fresh in-memory databases initialized by the same embedded
SQLx Migrator used in production.

After the first public release, this document must be revised before introducing append-only forward
migrations. Until then, no compatibility readers, repair migrations, or parallel schema paths are
maintained.

## Renderer boundary

The WebView has **no SQLite capability** and no `@tauri-apps/plugin-sql` dependency. It cannot execute `SELECT`, `INSERT`, `UPDATE`, `DELETE`, DDL, transactions, or arbitrary SQL.

Renderer code may only call typed Tauri commands with product/domain-shaped inputs. Rust owns:

- all database reads;
- all database mutations;
- transaction scope and ordering;
- parsing of internal JSON fields written by Knovum;
- schema migration and migration-history validation.

No Rust command accepts arbitrary SQL or an arbitrary runtime event as a database mutation mechanism.

Schema migration and feature access both use SQLx in Rust. No SQL plugin or migration command is
exposed to the WebView.

## Transaction rule

A business invariant spanning multiple statements must execute in one Rust SQLx transaction. Never reproduce a transaction by issuing multiple IPC commands and never move a generic transaction/SQL bridge into Foundation.

Model calls, network waiting and human approval must not hold a database write transaction.
Prepare their results outside the transaction, then recheck the publication conditions when committing.

Examples include Structure/Layout edits, content creation, subtree delete/restore, grounded messages,
linked two-Space movement and Space-package restore.

Read bundles that must represent one coherent point in time also use a Rust read transaction. Space
package export reads the Space row, Object Types, Saved Views, Relations, Nodes, Provenance,
Knowledge Events, Knowledge chunks, AI threads and AI messages inside one Rust
SQLx read transaction; the Renderer receives one complete manifest and never supplies authoritative
database state for export.

## Structure, camera and draft lifecycle

A workspace draft is an in-memory Space, not a persisted aggregate. The canvas host receives its
explicit `draft` or `persisted` lifecycle from the workspace. Draft camera changes and tab flushes
must not load or save camera state or materialize an empty Space.

The first durable action materializes the Space before committing Structure/Layout. Promotion shares
one readiness promise and retains the current surface. Opening a persisted Space loads its current
Structure/Layout snapshot and separate camera. The Renderer never saves an authoritative canvas
document: it projects an ephemeral Konva adapter document from the committed aggregate.

Tab transitions await admitted mutations and pending camera writes before leaving. Every structural
mutation carries `expectedEditRevision` and `mutationId`; Rust distinguishes an absent Space from a
revision conflict and records a typed receipt in the same transaction. Neither error permits forced
overwrite, automatic reload-and-retry, or a compatibility path.

## Media assets and derived analysis

Acquisition and analysis have different owners. Recording owns microphone sessions and ordered
chunk writes; file import owns source authorization and copying. Both construct their completed
assets through `assets::snapshot_to_imported`, with the actual file name, size, modification time
and content revision. A completed recording is not a synthetic path-only asset. Its IPC result is
validated as a ready audio asset; the Renderer does not invent missing identity fields.

Node insertion and updates synchronize durable enrichment intents inside the same Rust write
transaction. The normal node mutation boundary wakes pending work only after commit. Failed
dispatch has its own retryable Task Center entry and cannot roll back an already published file.
Pending import mutations do not dispatch until their acquisition is finalized.

`mediaAnalysis` owns decoded audio duration and waveform. Its strict current schema is
`{ schemaVersion: 1, sourceRevision, durationMs, waveform }`. Duration is a positive integer number
of milliseconds derived from the decoded 16 kHz mono sample count; waveform is a nonempty,
bounded sequence of finite peaks in `[0, 1]`. Silence is valid analysis, not missing analysis.

The `analyze_media` job requires the existing FFmpeg runtime, not Whisper, a speech model or an AI
connection. It publishes only `mediaAnalysis`, guarded by the current node source revision and
running job. Rust then records the dependent transcription intent. `mediaKnowledge` uses current
schema version `1`: source-bound revisioned transcript text, normalized segment/unit timing,
speaker tracks, subtitle cues and optional insights, but no waveform. Transcript
duration bounds transcript timestamps; it is not an alternative audio-card duration source.
Acquisition-supplied `durationSeconds` is source metadata, not the decoded audio-card authority.

The audio card consumes only analysis belonging to its current source revision. Unknown/stale
analysis displays an unavailable state and an unknown duration, never a fabricated zero-length
recording or a waveform copied out of an obsolete transcript. Pending, running, failed and retry
state remain in the durable jobs and Task Center rather than being mirrored into node metadata.

`media::prepared_audio` owns a disposable decoded-audio cache keyed by source revision and decode
policy version. Analysis and transcription reuse the same WAV/PCM outputs; they do not independently
decode the recording. Preparation publishes a completed scratch directory atomically and verifies
source identity before publishing its cache key. Existing startup analysis-cache cleanup removes
these disposable artifacts before work resumes. Cache paths are never persisted into node payloads
or exported packages, and a damaged cache is reported rather than treated as valid media.

Renderer submission uses the existing `p-limit` dependency to sequence projections per node, including
retries. It has no global batch-completion barrier: another node's long transcription cannot prevent
new audio analysis from being submitted. Rust retains the analysis/transcription/vision compute lanes.
The media engine and its cache do not depend on Space node types, model configuration or Vue state.

This is a pre-release contract change, not a compatibility upgrade. The existing job-table definition
and its checksum change; incompatible development databases are rejected before pool exposure. Old
media-knowledge payloads are not read, converted, or silently deleted. Use separate development data
when validating this branch; do not edit `_sqlx_migrations` to bypass validation. Original working
data must be backed up before any explicit development-data reset.

## Cross-resource cleanup

Separate SQLite databases and filesystem changes cannot commit atomically together.

Permanent Space deletion therefore treats the committed database mutation as authoritative and performs returned managed-file cleanup afterward. Cleanup failures are reported/logged as cleanup failures rather than pretending that the database transaction rolled back.

Credential deletion keeps an explicit read-backup/restore compensation path because the credential database and Spaces database have separate transactions.

## Builtin Agent state

Spaces SQLite owns conversations, branches, public messages, grants, Action receipts and Runs.
`spaces_conversations` and `spaces_conversation_messages` serve both built-in and external Agents.
Run kinds, authorization principals and knowledge actors distinguish `builtin_agent` from
`external_agent`; usage purpose uses `builtin_agent`. Model settings name the built-in profile
`builtinAgent`; message roles retain the standard `assistant` value. These current prerelease
baselines and strict settings schemas reject obsolete names without aliases or compatibility readers.
`spaces_builtin_agent_execution_segments` freezes the target/profile, execution revision, graph
version, directory digest and projection cursor. Tasks own stable input IDs, their acceptance and
control epochs. `spaces_builtin_agent_model_calls` records each admitted main/research/summary request,
usage certainty, checkpoint proof and explicit retry authorization. `spaces_builtin_agent_tool_operations`
relates durable SDK tool identity to existing Action invocation identity and its frozen business
route; no independent business result authority is created. Public AIMessage identities deduplicate
history projection in the same transaction as input acknowledgements and the cursor.

Known model consumption is recorded on its owning Run and ancestors even after cancellation or
budget exhaustion. Recording a paid receipt does not authorize publication or another request;
admission and business commit fences remain separate. Missing SDK usage remains unknown, never zero.

The SDK owns `<app-data>/agent-runtime/checkpoints.sqlite` through AsyncSqliteSaver. Knovum uses public
graph history, pending-write records and `adelete_thread`, never SDK SQL or private saver hooks.
The graph is versioned execution state, not exported product history. Credentials live only in
Rust/Python client memory and stdio execution context. Private workspace state belongs to a thread;
reviewed Skills and the Space index snapshot are read-only inputs. Persistent memory is ordinary
user-visible Space notes managed by explicit Actions. Completed superseded graph segments have a
30-day retention policy; unmatched orphan threads use the same grace period. Product records and
published resources retain their existing lifetime. Obsolete prerelease baselines fail explicitly.
