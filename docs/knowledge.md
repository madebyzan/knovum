# Knowledge State

This document is the current Spaces Knowledge contract. Knovum is still pre-release, so obsolete
development data may be reset and no compatibility readers or dual-write paths are maintained.

## Document extraction

PDF text is extracted with exact-pinned `pdf-inspector 1.20.0`, without its optional
PDFium or OCR features. Passages retain actual one-based page locators. Knovum no longer
renders PDF source pages in the evidence inspector; it shows textual evidence and opens
the original file through the existing OS integration.

File nodes retain one typed `documentCoverage` aggregate tied to `sourceRevision`:
parser identity, page/recognition coverage, source-owned content-addressed PNG resources, logical
occurrences, analysis provenance and explicit unsupported-format warnings. PDF visuals
use public positioned `ItemType::Image` items in `PositionFrame::Display`; normal text
pages do not generate page-level AI calls. Scanned, garbled or vector-text pages generate
one explicit recognition input instead of duplicate image inputs. Pages without extractable
content are reported separately, not automatically classified as scans or text-covered.
Inline images and vector graphics are not guaranteed to be fully covered by this API.
Core Graphics and Windows.Data.Pdf render the selected region directly into a bounded
bitmap. Coordinates are PDF points, lower-left/y-up in the rotated visible page frame;
Windows converts to top-left DIPs at 96/72.
Extracted raster paths contain both the managed source asset identity and raster digest.
Repeated images inside one document share a raster; different source assets own separate
files, so rollback of an unpublished import cannot remove another document's in-flight raster.
Raster geometry belongs to the explicit `pdfInput` contract; visual citations navigate to
the actual page and identify their occurrence, without redefining generic locator rectangles.
There is no alternate rendering engine or WebView
source-page command. Office parsing preserves supported embedded raster images and
their logical positions. XLSX cell APIs preserve actual addresses, formulas and cached
values without the vendor IR's 10,000-row cap. Spreadsheet chart appearance, exact image
cell anchors and legacy Office image positions remain explicitly unsupported.

Body text publishes independently of AI. Persistent `parse_document` and
`analyze_document` jobs use the existing enrichment queue. Missing visual configuration
is `awaiting_configuration`, not an execution failure. Invalid selected targets fail at
the Rust execution boundary instead of being swallowed as configuration deferral. Configuration changes resume
waiting jobs. Successful occurrences and raster paths are checkpointed, so retry skips
completed work. Provider failure does not trigger another engine. OCR and interpretation
become separate passages labelled `document_ocr` and `document_interpretation`, with
nearby source context and actual source positions. Grounded answers distinguish these
from original document statements. Visual file paths and operational progress do not
affect semantic revision identity; analysis content and provenance do.

Publication merges only owned fields into the current node in a write transaction,
rechecking source revision, node existence and running job identity. The central asset
reference walker includes document resources for packaging, rewriting and zero-reference
deletion; soft-deleted nodes retain resources for recovery. Unpublished/obsolete files
are reclaimed by the existing startup orphan collector, not a collector racing imports.

Budgets: one parser at a time across imports/jobs; input 128 MiB; 2000 PDF pages;
4 Mi UTF-16 body units; Office ZIP expansion 256 MiB/20,000 entries;
2000 visual resources/occurrences, 256 MiB normalized rasters, 32 MiB/24 MP per image,
4 Mi UTF-16 analysis units; native raster long edge 2048 pixels and scale at most 2.
Excess fails visibly. PDF page count is checked after parsing and is not an internal
decompression-work limit. Native rendering and visual calls are serial. Native rendering checks cancellation between operations; model calls and desktop Agent sessions support cooperative cancellation, with ACP acknowledgement before session teardown.

## AI task results and execution selection

Knowledge tasks share product execution policy with the Assistant. A configured and
explicitly selected desktop Agent can execute transcript insights, image understanding
and document visual understanding with zero user model connections and no Knovum-held
API key. Its tools/Skills must actually satisfy the task; execution failure is visible
and never silently invokes a Provider using another account.

Transcript insights consume existing timestamped transcript text. The existing bounded
chunking, candidate merge and chapter validation are retained: chapter starts must be
unique transcript segment timestamps, begin with the first segment and end within the
transcript duration. Strict JSON and nonempty bounded summary/chapters are required.
A failed/invalid/cancelled insights task never deletes or replaces the transcript.
Publication checks source identity and the complete original media-knowledge snapshot
inside the existing node/chunk write transaction, protecting concurrent manual edits.

Image understanding prepares the current applied `imageEdit` as a canonical PNG from
the authorized managed original, verifies its byte revision, and binds knowledge to
the rendered revision. Video visual enrichment currently interprets only `previewPath`:
it is preview-image understanding, not whole-video analysis. Document visual delegates
receive the verified materialized raster for the exact occurrence plus its nearby
context; original parser locations, resource IDs, source revisions and per-occurrence
checkpoints stay authoritative. Body parsing and basic resource use remain local and
do not require any AI selection. Cancellation, job liveness, source identity and document
coverage edits guard publication; successful Agent results enter the same knowledge
passages and indexing as model results.

`executionSource` records either actual Provider/model identity or the external Agent
runtime kind. Agent execution does not fabricate model identity, tokens or cost.
Local transcription continues to use its own Whisper resources, media preparation,
FFmpeg and cancellation registry; it never resolves user AI model settings or keys.
Missing local model resources/tooling/source bytes are separate failures from an
unselected AI executor. This change adds no speculative external long-audio ASR path,
whole-video inference engine or hosted subscription service.

## Document locations and job checkpoints

Office body, header and footer locations use the library's public logical positions; they do not infer rendered pagination. XLSX images and text shapes use public worksheet interfaces and retain EMU geometry. Geometry is not the original cell anchor. Legacy binary Office image locations remain limited by the parser's location support.

Intermediate document visual results belong to the enrichment job and are stored per occurrence in `spaces_document_visual_checkpoints`, not as a duplicate canonical Knowledge store. Completed analysis is published on job completion or failure, after checking that the node document-coverage snapshot has not been edited concurrently. Recovery fills unfinished occurrences and preserves already published completed knowledge, including edits made before recovery. Cancellation retains checkpoints, but unpublished resources of cancelled jobs are not reclamation roots. Retryable jobs for the current source remain referenced by the asset graph.

## Database migrations

`spaces.db` is created from the single pre-release SQLx v1 baseline at
`src-tauri/migrations/spaces/0001_spaces.sql`. SQLx records its version and checksum in
`_sqlx_migrations`; there is no parallel `user_version`, schema-contract table, repair table, or
compatibility reader. Schema changes are folded into the baseline while the product remains in
pre-release development, and obsolete development databases are deleted explicitly. The baseline
contains the complete document job, visual checkpoint and derivation source snapshot contracts.
There are no duplicate resource tables: the bounded document manifest belongs to the existing node aggregate.

## Authoritative Knowledge revision

Semantic Node `knowledgeRevision` is content-derived authority rather than caller-owned metadata. Vue computes the same `node1:` revision for immediate UX, while Rust recomputes it from the canonical semantic projection and rejects a mismatched declared revision at the durable boundary. Local image resource paths are excluded from Note revision identity; semantic non-image `src` values remain included. Canonical object-key ordering is stable across TypeScript and Rust.

Derived Knowledge is generated from Node content in Rust. Durable supplemental passages have bounded UTF-8 size and explicit origin revision domains. Canonical `node:<id>` chunks remain bounded metadata records.

## Historical grounded evidence

Grounded citations have explicit `current` and `detached` states. A current citation carries live Node/chunk IDs and validates against the persisted thread scope and current Knowledge. A detached citation is an immutable historical snapshot: quote, locator, source revision and title remain while live Node/chunk IDs are absent. `all-spaces` threads are read-only retrieval contexts: citations from the current Space may remain current, while citations from other Spaces are persisted immediately as detached snapshots and never become cross-Space Knowledge ownership.

Content replacement, semantic-revision changes, Move and Trash detach affected local live citations instead of deleting evidence or silently rebinding it to newer content. Scope membership is pruned when a live Node leaves a Selection/Section scope. Canonical citations whose stable `node:<id>` chunk changes semantic revision are detached before mutation. Move does not rewrite historical cross-Space snapshots.

## Derivation lineage

Knowledge Derivation is a resource-transformation contract, not a synonym for Provenance. A Derivation record captures the immutable execution facts for an attempted or completed transformation: typed input Node IDs and source revisions, action identity/version, bounded non-secret parameters, engine/model identity, actor, timestamps, status and output Node IDs. Completed outputs additionally receive semantic `derives-from` Relations to their inputs.

Provenance remains epistemic evidence attached to a Knowledge Node or Relation: it identifies the source revision, locator and quote that support a claim. It must not be overloaded with model parameters, media pipeline state or artifact-generation metadata.

Artifact creation and Knowledge acceptance are separate transitions. Deterministic transformations and AI-generated documents or media create usable derived resources directly. Provenance, exact source revisions and generated identity remain visible; none of these imply factual verification. Durable Knowledge mutations use a closed KnowledgeChangeSet contract that Rust validates and commits atomically.

Creation of a completed output, its Derivation record and its `derives-from` Relations is one durable transaction. A cancelled or failed attempt may retain an operational Derivation record for inspection, but it must not create a completed output identity or Relations to a nonexistent artifact. Derived source revisions are never silently rebound when an input later changes; rerunning creates a new Derivation and output revision or resource according to the action contract.

## `.knovumspace` package contract

`.knovumspace` exports and imports use package version `1` only. Import rejects any other package version; there is no legacy package upgrade chain. Current and detached citations are validated against the current schema before restore.

Every import receives an isolated UUID asset namespace so the same backup can be restored repeatedly without target collisions. Asset indexes carry SHA-256 content revisions. The Rust import verifies materialized asset bytes/directories against the index and requires the index token set to match assets referenced by packaged Nodes. Managed Node `sourceRevision` must agree with the indexed digest. Export performs the inverse preflight before writing the archive.

Manual export writes a same-directory staging archive and replaces the selected destination only after ZIP completion. Replacement keeps a rollback copy if the final rename fails. Export and automatic backup validate the generated v1 manifest before ZIP creation.

## Search and passage work

The renderer passage splitter uses linear UTF-8 width accounting and the same preferred whitespace window as the Rust splitter. Rust remains the durable authority for derived passage generation and all persisted passage size/revision validation. SQLite owns current Knowledge chunks and index publication markers; LanceDB stores embedding vectors. Semantic search validates LanceDB matches against current SQLite chunk identities and revisions before hybrid ranking. All-Spaces Ask reuses the existing per-Space semantic search and global lexical search; it does not introduce a second global Knowledge store.

Embedding computation runs outside the SQLite write transaction. Publication acquires the existing `BEGIN IMMEDIATE` boundary, revalidates the chunk/source revisions, drains pending retrieval invalidations for the affected Spaces, atomically upserts LanceDB vectors by chunk ID, and commits SQLite markers in that order. Chunk replacement/deletion and outbox draining use the same write boundary, so neither can invalidate a validated chunk during publication or remove freshly published vectors through an older invalidation. Successful invalidation deletes both vectors and their SQLite markers.

SQLite and LanceDB do not share a distributed transaction. A failed vector upsert preserves previously committed rows and does not publish new markers; chunks without committed markers remain eligible for indexing. Durable invalidations are acknowledged only after vector deletion succeeds. Search still rejects stale revisions; foreign keys remain enabled, and source data is never reset to repair an index.

## Module boundary

Knowledge State does not introduce a separate compatibility layer or speculative ANN subsystem. Domain truth stays in revision, passage, citation, asset and package boundaries rather than renderer fallback logic. MCP writes enter long-term Knowledge only through the same validated atomic KnowledgeChangeSet use case as native product actions.
