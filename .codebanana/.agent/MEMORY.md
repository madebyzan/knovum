# MEMORY.md — knovum (Coding Session)

## Rules (user-set)

- Never run cargo in the sandbox; Rust is verified locally by the user (`pnpm check:rust`). Standalone `rustfmt --edition 2021` is fine.
- No fallback/compat patches, no meaningless code; DRY/SOLID expected; solve at the architecture level, then commit + push without step-by-step confirmation. "我不关心过程，只在意结果".
- Node bar contract: `[primary][capabilities…][…] | [details][export]` — nothing ever after the trailing access group.
- Pre-release single v1 baseline migration: edit `src-tauri/src/spaces/db/migrations/0001_spaces.sql` directly (developer must delete local `spaces.db`). **From first public release the baseline is frozen and every change ships as the next numbered migration** — an installed app must never ask the user to delete their library (`docs/data-ownership.md` §Migration policy).
- Schema invariants enforced by `spaces/db/tests`, not review: every FK is indexed (partial indexes don't count); every `space_id` participates in a FK; CHECKs state invariants, never catalogue membership (model/engine/action ids live in Rust); a stored identity (producing run, promoted voice) is deliberately not a reference.
- No duplicate entries across surfaces: voices live only in Settings; a derived node owns playback/export, panels must not repeat it.
- Error presentation: keep the existing architecture (raw error codes surfaced as-is); global error-code localization is deferred — don't raise or patch per-feature mappings.
- Extensions are single-account: one authorization per connector (identity = connector_id); never reintroduce account selection/binding.

## Architecture facts

- Edit history is row-scoped, not state-scoped: `spaces_edit_history.patch` (single JSON column) holds before/after for each changed structure row (`repository::rows::StructureRow` = node+layout), relation style, relation lifecycle and node presence. `history/patch/` is pure (diff + side + content-size neutrality), `history/replay.rs` + `history/group/` orchestrate, `repository/rows.rs` owns all row-scoped SQL (`vacate` → `write`, because `spaces_structure_active_order` is unique among live rows; `erase` for rows that leave a Space in a transfer). A replay revalidates via `repository::load_in_transaction` and bumps `repository::advance_revision`. Node presence is derived from the snapshot's content refs (coverage invariant), never re-read. Measured: 3000-node Space, 100 retained edits 170MiB → 63KiB, 40ms → 0.03ms per edit, flat in Space size.
- Reserved outputs (`derivation/pending_output.rs` + table `spaces_reserved_outputs`): every Run reserving a canvas output does it in the Run-creation transaction (`reserve_in_transaction`), with placement + `derives_from`. `release` on settle (inside `persist_output_node`), `settle_failed`/`settle_abandoned` when a Run ends unpublished (wired into `spaces/run_execution` after_attempt/after_controls hooks). Speech entry point: `speech/output_reservation.rs` (`OutputContext`), node ids `{run_id}-output` / `{run_id}-vocals|background`.
- Derived-node inheritance declared once in `derivation/inheritance/` (`DerivedOutput::keeps` → Facets Labels/Timeline/Sound/Speech/Picture); every publisher calls `inherit` after its file settles.
- One published-asset settlement: `assets/published_asset.rs::PublishedAsset::settle_onto` (path/fileName/revision/size/modifiedAt/cover + clears processingState). Covers: `previews::generate_for_published_asset[_off_thread]`, `GeneratedPreview::settle_onto`, discard via `derivation::video_output::discard`.
- Docs contracts: `docs/creation-sources.md` (§Reservation/§Settling/§Inheritance), `docs/spaces.md`, `docs/ui-system.md`.
- Tooling: pnpm via `corepack pnpm`; checks = typecheck, oxlint, oxfmt, vitest, depcruise, jscpd (knip OOMs in sandbox).
