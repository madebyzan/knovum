# IPC remediation — single-agent execution

## Request and constraints

Systematically improve the existing Tauri IPC boundary, with meaningful code only,
no meaningless aliases, no compatibility readers, and verified code quality.
The latest user instruction requires one agent; it overrides multi-agent workflow
review and execution. Earlier child work has stopped. No new dependency is explicitly
requested; retain the current Tauri and ts-rs toolchain.

## Evidence

- `foundation/ipc/invoke-command.ts` normalizes native rejection values but branches
  only to preserve native argument arity. It does not provide command-signature typing.
- Most callers use `invokeCommand as invoke`, hiding the actual error boundary.
- Video-editor presentation imports the transport directly for fonts/proxy preparation.
- Rust owns video-edit requests/progress/fonts; frontend repeats their wire structures.
- Existing ts-rs generation and model schemas already provide DTO generation and
  runtime validation. Mutation receipt confirmation belongs to Spaces, with regression tests.
- Native invoke accepts strings and caller-supplied return types:
  https://v2.tauri.app/reference/javascript/api/namespacecore/#invoke
- Serialized Rust errors are deliberate wire contracts:
  https://v2.tauri.app/develop/calling-rust/#error-handling
- ts-rs derives DTO declarations and supports per-type export paths:
  https://docs.rs/ts-rs/12.0.1/ts_rs/trait.TS.html

## Principles and drivers

Keep Tauri as the transport; one small error boundary; capability-owned operations;
Rust-owned wire DTOs with explicit validation; no global service or handwritten command map.
Drivers: correct ownership, source-of-truth integrity, small verifiable changes.

## Options / decision

1. Keep ts-rs and capability operation functions, repair presentation ownership and
   duplicate wire contracts. Uses existing tools, preserves runtime/security contracts;
   command signatures still require review/tests rather than generated static pairing.
2. Adopt full command binding generation. Can statically pair names/args/results, but
   requires selecting a new dependency and migrating the existing DTO generation,
   namespaced plugins and Channels. No dependency addition is explicitly requested.
3. Handwrite a global CommandMap or custom Rust parser/generator. Rejected: a second
   signature source of truth or substantial custom infrastructure duplicates upstream work.

Choose option 1 as a complete local boundary remediation, not a claim of end-to-end
generated command typing. Do not install a new framework or add compatibility adapters.

## Cleanup sequence

1. Expand regression coverage before production edits: forwarding results/arguments/
   options, known and malformed error payloads; retain mutation-confirmation tests.
2. Remove transport aliases across all production callers. Simplify forwarding with
   the native parameter tuple, preserving argument count and object/Channel identity.
   Remove unused identity-only error type aliases.
   Import AI completion wire types under their Rust-generated names without alias re-exports.
3. Move font/proxy calls behind video-editing capability operations. Export actual Rust
   request/progress/font/derivation-output DTOs through existing ts-rs. Preserve the
   stricter frontend model schemas; use them to validate returned node data.
4. Enforce that presentation/components cannot import the raw app command boundary;
   enforce native invoke ownership using existing lint tools. Keep official plugin APIs
   and Channel imports available. Document ownership and honest typing limits in the
   authoritative architecture/dependency docs.
5. After all remediation, run contract generation, formatting, focused regression tests,
   check:web and check:rust. The latest user instruction defers all checks/formatting until
   edits are complete and prohibits packaging; do not run web/desktop builds or packaging.
   Fix failures caused by the change. Report unrelated pre-existing failures explicitly.

## Acceptance / verification

- No `invokeCommand as invoke` imports remain; no redundant CommandErrorPayload alias.
- Native argument arity and reference identity unchanged; normalized errors preserve codes/causes.
- UI/presentation owns no raw app command strings/imports; capability functions expose
  request types and concrete return types. Fonts/proxy/video progress use Rust exports.
- Returned derivation data is validated before entering the typed UI model.
- Mutation rejection/ambiguous transport semantics unchanged.
- Lint/static dependency rules reject a probe violating the new boundary.
- Existing frontend/Rust checks pass, or baseline blockers are evidenced. No packaging/build.
- No new dependencies, alternate transport, compatibility layer, or unrelated-file rewrites.

## Risks / stop condition

Aliases span many callers: mechanically change only imported identifier references;
typecheck and regression tests verify integration. Generated exports may uncover stricter
wire/model differences: validate and translate at the capability boundary, never cast.
Stop when the above criteria are evidenced and the diff has been reviewed by this agent.

## Execution notes

- Baseline regressions passed before the user requested deferred checks: 15 IPC/
  mutation/project tests and 4 proxy lifecycle tests.
- Latest user constraints: all subsequent tests/checks/formatting run after remediation;
  no packaging or application build. Contract generation uses the existing ignored Rust
  export test as its code-generation entry point; it is not an application build/package.
- 72 production imports were renamed with TypeScript symbol-aware edits.
- Feature font queries and proxy lifecycle now own their transport, cache/validation and
  cancellation responsibilities. No global command service or thin per-command wrappers
  were introduced.

## Verification evidence

- `pnpm contracts:generate`: passed; Rust owns 15 additional generated wire DTOs.
- Negative ownership probes: Oxc rejected a raw native invoke import; dependency-cruiser
  rejected a presentation import of the app command boundary. Temporary probes removed.
- Frontend typechecks (application, tests, node tooling, e2e), Oxc and dependency-cruiser:
  passed. Dependency graph: 1255 modules, 3884 edges, no violations.
- Full Vitest suite: 143 files / 514 tests passed. Includes 36 focused IPC, mutation,
  video-project, font-query, proxy-lifecycle and render-contract regressions.
- Knip: passed. jscpd: 0.44% duplicated lines, below the existing 1% gate.
- The initial `pnpm check:web` passed every code gate and flagged only this plan's
  Markdown formatting. After formatting the plan, `pnpm format:check` passed on all
  1265 matched files. No production code changed after its successful verification.
- `pnpm check:rust`: exit 0. Capabilities, rustfmt and Clippy with warnings denied
  passed; 239 unit tests and 1 resource smoke test passed. The ignored contract-export
  test was separately executed successfully by `pnpm contracts:generate`.
- No package/dependency manifest or lockfile changes; no application build/packaging.

## Completion

The requested boundary remediation is complete under single-agent execution. Reviewed
the final error boundary, capability ownership, generated DTO paths and consumed-value
validation. No temporary ownership probes or pending implementation remain. Full
command-signature binding generation remains a separate dependency decision; the current
transport intentionally does not claim static command-name/argument/result pairing.
