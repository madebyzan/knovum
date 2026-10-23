# Local data ownership

SQLite is a storage primitive, not a shared application service.

## Rule

Spaces owns its persisted domain data, schema, Rust repository and tests. Do not create a global `DatabaseService` or expose arbitrary SQL across IPC.

The application database is `spaces.db`. SQLx owns its migration history in
`src-tauri/src/spaces/db/migrations/` and records every applied version and checksum in `_sqlx_migrations`.

The database resolves from Tauri's AppConfig directory through the connection policy in `src-tauri/src/sqlite.rs`. Spaces repositories own their SQL and domain rules.

## Migration policy

Knovum-owned data formats, packages, private protocols, runtime manifests/descriptors and the Agent
profile/graph identity start at version 1 with their current fields. Dependency and external API
versions remain upstream-owned. Record revisions and Run state versions still advance normally.
Store, package and protocol envelopes remain separate contracts with their own explicit versions.
Spaces conversations contain no product mode. The current Space package is v1 and the private Agent
RPC is v1.

The embedded SQLx migrator runs before the Spaces pool is exposed and installs each pending
migration transactionally.

**0.0.1 is published, so the shipped schema is frozen.** `0001_spaces.sql` is not an editable file
any more: it is the schema already recorded in other people's `_sqlx_migrations`, and editing it
changes a checksum they have on disk. Every schema change ships as the next numbered migration
(`0002_…`, `0003_…`) in the owning feature's migration directory — `spaces/db/migrations/`,
`extensions/migrations/`, `ai/usage/migrations/` and `secrets/migrations/` each keep their own
history.

An installed application must never ask the person using it to delete their library. "Back up and
reset it" is not a migration path, and neither is a release note explaining why their Spaces are
gone. A change that cannot be expressed as a forward migration is not ready to ship.

The freeze covers comments and whitespace: the header line of `0001_spaces.sql` still describes the
schema as pre-release, and it stays wrong, because SQLx checksums the whole file and a tidier
comment would lock every 0.0.1 library out of the app.

The two migrator failures therefore mean different things and are tested separately in
`spaces/db/tests`:

- a checksum that no longer matches an applied migration (`VersionMismatch`) means a shipped
  migration was edited. That is a developer mistake and must be caught before release, never by a
  user whose library suddenly refuses to open;
- a recorded version the binary does not carry (`VersionMissing`) means the database was written by
  a newer release. The older binary refuses to touch it and says so, instead of downgrading data.

Neither case resets or rewrites user data. Running the migrator against an up-to-date database is a
no-op, which is what makes an additive migration safe to install.

A forward migration adds; it does not reinterpret. New tables, columns with defaults, indexes and
triggers are additive, and a new column that existing rows must answer for is backfilled inside the
same migration from data already present — not left as a sentinel every reader has to interpret.
SQLite cannot alter a `CHECK` or add a foreign key in place, so a constraint change is a documented
table rebuild inside one migration (create, copy, drop, rename, recreate indexes and triggers),
never a compatibility reader beside the old shape.

Dropping a column or a table deletes someone's content, so it is a product decision with its own
review, not a cleanup step inside a schema change. Renaming is the same decision in two parts.

Fresh-database tests do not prove an upgrade. A migration is accepted with a test that starts from
the previously shipped schema, carries representative rows through the new migration and asserts
both the resulting schema and the surviving data; `docs/release.md` keeps the packaged upgrade pass
that runs the full history against a real installed library. Immutability itself is a CI gate: the
Quality workflow fails a pull request that modifies, deletes or renames a migration already present
on its base branch.

Install the matching SQLx CLI with
`cargo install sqlx-cli --no-default-features --features sqlite --version 0.8.6 --locked`.
`pnpm check:rust` runs Rust tests against fresh in-memory databases initialized by the same embedded
SQLx Migrator used in production.

## Schema invariants

These hold for every table and are enforced by tests in `src-tauri/src/spaces/db/tests/`, not by
review:

- **Every foreign key is indexed.** Without an index on the child columns SQLite scans the whole
  child table for each parent row it deletes or nulls, so removing one Space or one node degrades
  with the size of the library. A partial index cannot serve a foreign key, and only a
  leading-column match counts.
- **Space-scoped rows belong to their Space.** A table with `space_id` carries it in a foreign key,
  directly or through a composite key into another Space-owned table. Otherwise deleting a Space
  leaves rows no feature can reach and no reclamation pass can drain.
- **Constraints state invariants, not catalogue membership.** Row state machines, structural shape,
  JSON validity, identity and physical ranges belong in the schema. A model, engine, action or
  aspect-ratio identifier is catalogue data owned by Rust: listing those values in a `CHECK` makes
  every new model a table rebuild, and the edge already validates them.
- **A stored identity is not a reference.** Columns that record which execution produced a row, or
  which voice a candidate was promoted into, outlive the Run or profile they name and say so in the
  schema. Everything else that points at a live row has a foreign key.
- **Cleanup direction decides the key.** A retained-reference table says "keep this file": it must
  cascade with its owner, or the reclamation sweep treats the file as still in use forever. A
  tombstone table says "delete this thing": it must survive its owner, or the instruction is erased
  together with its subject and the vectors or files it named are never removed.
  `spaces_speech_asset_publications` is the first kind; `spaces_retrieval_invalidations`, which
  drives LanceDB deletion, is the second and is listed as such in the schema tests.
- **Only indexed columns may rewrite an index.** A trigger without a `WHEN` guard reindexed every
  sibling title when a Space was reordered. An index-maintaining trigger states which columns it
  reacts to.
- **Anything that only serves a replay is bounded by the replay window.** `RETAINED_EDITS` bounds
  undo depth, and the edit receipts that make a mutation idempotent are bounded by the same window:
  a receipt can only be dropped once a hundred later edits have made replaying it meaningless.
- **A full-text index is addressed by row, never by a column beside the text.** FTS5 has one key,
  its rowid, and supports no secondary index: a trigger that deletes `WHERE chunk_id = OLD.id`
  reads the whole index for every row removed, which made deleting one Space's knowledge quadratic
  (measured: 1600 chunks took 4.3 s, one chunk in a 24 MiB index 6.4 ms). The maintaining triggers
  mirror the owner's rowid into the index and address rows by it. `VACUUM INTO` may renumber the
  rowids of a table whose primary key is not an integer, so a database snapshot rebuilds both
  indexes in the copy and validation refuses a copy whose index no longer answers for its own rows.
- **A record of a change is proportional to the change.** History stores the rows an edit touched,
  never the state it produced. A whole-state record makes every edit cost the size of the Space, so
  a large Space would pay megabytes per keystroke and keep a hundred copies of itself; it also
  overwrites work it never meant to touch when replayed. `spaces_edit_history.patch` holds the
  before and after value of each changed structure row, layout, relation style, relation lifecycle
  and node presence, and a replay writes exactly those back and then revalidates the Space it
  produced through `repository::load_in_transaction`.
- **A streamed record costs one commit per batch, not one per chunk.** An Agent answer arrives as
  hundreds of chunks of a few dozen bytes. A transaction per chunk spends a commit on each one and
  rewrites the accumulating assistant message from its start, so the cost grows with the number of
  chunks rather than the size of the answer (measured: a 129 KiB answer in 4000 chunks took 51 s
  and rewrote 252 MiB; recorded in batches it takes 1.3 s in 126 commits). `spaces::runs::stream`
  records whatever has queued in one transaction, in arrival order, and the screen follows the
  stream itself rather than its durability. Streamed updates are never recorded from independently
  spawned tasks: `spaces_run_events.sequence` is the order the conversation is read back in.

## Storage settings

`src-tauri/src/sqlite.rs` owns the pragmas every feature database shares. WAL is paired with
`synchronous=NORMAL`, the mode WAL is designed for: commits stop waiting on an fsync, the file
stays safe from corruption, and the exposure is losing the last transactions to a power cut, which
local recovery archives cover. The cache and mmap windows keep index pages of a growing library in
memory, and temp tables stay in memory so the sorts query plans ask for never touch the filesystem.

`PRAGMA optimize` runs once each feature database is migrated. Without statistics the planner falls
back to heuristics and can choose a Space-wide index where a selective one exists.

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

A workspace draft is an in-memory Space, not a persisted aggregate: it has no `spaces` row, so no
Space-owned row can be written for it. The Space store answers `spacePersistence(spaceId)` once, and
every Space-scoped writer reads that answer instead of re-deriving it. Draft camera changes and tab
flushes must not load or save camera state or materialize an empty Space; a draft's conversation
composer state stays in memory and is written once the Space exists. A tab flush that wrote for a
draft would fail on the missing row and abort the whole transition, leaving the workbench unable to
leave the empty Space.

The first durable action materializes the Space before committing Structure/Layout, and submitting a
conversation materializes it before the conversation is created. Promotion shares
one readiness promise and retains the current surface. Opening a persisted Space loads its current
Structure/Layout snapshot and separate camera. The Renderer never saves an authoritative canvas
document: it projects an ephemeral Konva adapter document from the committed aggregate.

Tab transitions await admitted mutations and pending camera writes before leaving. Every mutation
carries `mutationId`; Rust records a typed receipt in the same transaction. A structural edit that
expresses intent against the rendered structure — `execute_space_edit`, undo/redo, clipboard paste,
cross-Space transfer and JSON Canvas import — also carries
`expectedEditRevision`, and Rust distinguishes an absent Space from a revision conflict. Content
creation (`create_nodes`) and discarding one failed pending creation (`discard_pending_content`)
carry no caller revision: they are node-scoped appends and removals whose base is the structure
loaded inside their own write transaction, which SQLite serializes against every other writer. A
parallel import or an agent-written node must not make the next local creation fail. No error permits
forced overwrite, automatic reload-and-retry, or a compatibility path.

A loaded projection is applied to the session only when it is not older than the snapshot the
session already holds: parallel creations read their committed projection after their own commit, so
a slower read must not reinstate a Space that another creation has already moved past.

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

The media-knowledge job contract is v1 and shipped. Changing it is a forward migration over the
existing job rows, not a new checksum for the same baseline: jobs people already have are carried
over or retired by that migration. Old media-knowledge payloads are never silently reinterpreted,
and `_sqlx_migrations` is not edited to bypass validation.

## Cross-resource cleanup

Separate SQLite databases and filesystem changes cannot commit atomically together.

Permanent Space deletion therefore treats the committed database mutation as authoritative and performs returned managed-file cleanup afterward. Cleanup failures are reported/logged as cleanup failures rather than pretending that the database transaction rolled back.

Credential deletion keeps an explicit read-backup/restore compensation path because the credential database and Spaces database have separate transactions.

## Execution ledger retention

`spaces_runs`, `spaces_run_steps`, `spaces_run_attempts`, `spaces_run_events`,
`spaces_knowledge_events` and `ai_token_usage_events` are **retained for the life of the library**.
There is no retention job and no age cutoff.

They are provenance, not telemetry: the Run is the answer to "what produced this node", the
knowledge events are the answer to "who changed this and when", and the usage ledger is the answer
to "what did this cost me". Deleting them to save space would delete the user's own history, which
is exactly what local data ownership exists to prevent.

The cost is measured and bounded. A six-step workflow Run with its attempts and events occupies
**13.7 KiB** including indexes, so fifty Runs a day for a year is 245 MiB — an order of magnitude
less than the media assets the same year of work produces. A knowledge event costs **183 bytes**
and a token usage event **122 bytes** including indexes, so twenty thousand of either is under
4 MiB. A streamed Agent answer records about 2.7× its own text.

Deletion remains driven by ownership, not by age: the ledger of a Space leaves with the Space
through `home_space_id`, and the Runs of an external conversation leave with the conversation.

## Builtin Agent state

Spaces SQLite owns conversations, branches, public messages, grants, Action receipts and Runs.
`spaces_conversations` and `spaces_conversation_messages` serve both built-in and external Agents.
`spaces_conversation_drafts` owns unsent prompt text, current managed attachments and extension
selection per `(space_id, conversation_id)`. Rust validates node revisions and managed asset paths
before writing or restoring a draft. A directly selected external file is never restored from its
old path: only its display name is retained so the composer can require an explicit picker or drop
authorization again. Sending remains blocked until each retained reference is reattached or explicitly removed; picker
callbacks belong to the current draft lifetime, including reopening the same conversation. Serialized
attachment input is bounded before persistence and before decoding disk data. Draft commands remain domain-shaped; the WebView never receives SQL access.
Run kinds, authorization principals and knowledge actors distinguish `builtin_agent` from
`external_agent`; usage purpose uses `builtin_agent`. Model settings name the built-in profile
`builtinAgent`; message roles retain the standard `assistant` value. These shipped baselines and
strict settings schemas carry no aliases or compatibility readers; a renamed value is a forward
migration over the stored rows, decided like any other schema change.
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
published resources retain their existing lifetime. An incompatible graph identity retires its own
execution segments, which is disposable execution state and never user content.

## Local recovery archives

The application backup capability composes feature-owned `spaces.db`, `extensions.db`, `ai-usage.db`,
strict Tauri Store settings, Space managed assets and installed Agent extension files. It does not
become a database owner: SQLx baselines and Space quiescence checks remain authoritative. Unsent
conversation drafts are included; active or replayable execution is not admitted.

Backups are local recovery archives for the original canonical storage roots. A versioned manifest
records those roots, exact baseline fingerprints, declared files and their hashes. Restore publishes
validated roots only during exclusive startup maintenance and reconciles its publication journal
before any feature storage opens. The credential database and its sidecars are excluded from archives
and cleared on successful restore so restored identifiers cannot inherit current credentials.
Agent SDK checkpoints, models, runtimes, caches, logs and disposable UI sessions are outside the archive.
