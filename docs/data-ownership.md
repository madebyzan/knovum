# Local data ownership

SQLite is a storage primitive, not a shared application service.

## Rule

Spaces owns its persisted domain data, schema, Rust repository and tests. Do not create a global `DatabaseService` or expose arbitrary SQL across IPC.

The application database is `spaces.db`. SQLx owns its current v1 baseline in
`src-tauri/migrations/spaces/` and records the installed version and checksum in `_sqlx_migrations`.

The database resolves from Tauri's AppConfig directory through the connection policy in `src-tauri/src/sqlite.rs`. Spaces repositories own their SQL and domain rules.

## Migration policy

Before the first public release, Spaces owns one SQLx v1 baseline. Schema changes update that
baseline and require deleting obsolete development databases. The embedded SQLx migrator runs
before the Spaces pool is exposed, installs the baseline transactionally, and rejects checksum
divergence. Store/package/protocol envelopes remain separate contracts and use version `1` during
pre-release development.

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

Examples include Space node batches, soft-delete/restore, position batches, grounded messages,
cross-Space movement and Space-package restore.

Read bundles that must represent one coherent point in time also use a Rust read transaction. Space
package export reads the Space row, Object Types, Saved Views, Relations, Nodes, Provenance,
Knowledge Events, Knowledge chunks, AI threads and AI messages inside one Rust
SQLx read transaction; the Renderer receives one complete manifest and never supplies authoritative
database state for export.

## Cross-resource cleanup

SQLite cannot atomically commit filesystem or Keychain side effects.

Permanent Space deletion therefore treats the committed database mutation as authoritative and performs returned managed-file cleanup afterward. Cleanup failures are reported/logged as cleanup failures rather than pretending that the database transaction rolled back.

Credential deletion keeps an explicit read-backup/restore compensation path because a Keychain secret can be restored if the following database deletion fails.
