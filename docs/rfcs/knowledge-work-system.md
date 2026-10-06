# Knowledge Work System

- Status: Dated implementation baseline; product targets have separate acceptance
- Decision date: 2026-09-21
- Primary design source: `DESIGN.md`

This RFC records the implemented contract distilled from the approved Knovum final design. It
replaces earlier partial Agent, derivation and workflow directions. The source modules, baseline
schema, generated contracts and quality gates are the executable evidence for this contract.

## Implementation-state map

This RFC is a dated decision record, not a second current architecture specification. [architecture.md](../architecture.md), [system-architecture.md](../system-architecture.md), [knowledge.md](../knowledge.md) and [spaces.md](../spaces.md) own current contracts. [DESIGN.md](../../DESIGN.md) owns product goals.

| Area                                | Implementation evidence                                                                  | State / acceptance boundary                                                                                            |
| ----------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Space actions and authorization     | `src-tauri/src/spaces/actions`, `authorization`                                          | Implemented domain and atomic authorization paths; scenario tests own behavioral evidence.                             |
| Durable Runs and observation        | `src-tauri/src/spaces/runs`; `src/features/spaces/runs`                                  | Implemented persistence/Channel projection; Task Center does not own execution.                                        |
| Workflows                           | `src-tauri/src/spaces/workflows`; `src/features/spaces/workflows`                        | Implemented definitions and execution; contextual capture/editing goals in DESIGN are not blanket completion claims.   |
| Builtin Agent and external tools    | `src-tauri/src/spaces/conversations`, `mcp`; `src/features/spaces/conversation`          | Implemented runtime/conversation APIs; unified product presentation and Settings targets require their own acceptance. |
| Derivation and resource publication | `src-tauri/src/spaces/derivation`, `assets`; `src/features/spaces/knowledge-derivations` | Implemented revision/lineage boundaries; action-specific completeness is evaluated independently.                      |
| Cross-platform desktop UX           | `docs/platform-matrix.md`, `docs/desktop-foundation.md`                                  | Requires actual macOS/Windows evidence; source presence alone is not platform certification.                           |

## Product boundary

Knovum is a local-first knowledge work environment. Resources remain useful without an Agent,
subscription or official account. Human actions, fixed AI operations, Workflows, the built-in
Builtin Agent and external Agents reuse the same Space-owned business rules.

The only excluded product capability is official subscription, payment and hosted inference quota.
Self-configured models, the Builtin Agent, external connections, Workflows, durable Runs,
authorization and recovery are part of the domain baseline; their individual product surfaces and acceptance are tracked below.

## Stable language

- `SpaceAction`: a versioned business capability with typed inputs, outputs, effects and execution
  requirements.
- `Tool`: a protocol projection of a SpaceAction or external capability; never the policy owner.
- `Command`: a typed Tauri IPC or application-command entry point.
- `Invocation`: one authenticated SpaceAction call.
- `Receipt`: the authoritative result of committed side effects.
- `Run`: a durable long-running business execution.
- `Job`: a process-local engine activity, never a Workflow or Builtin Agent identity.
- `Workflow`: a user-owned, publishable typed program.
- `Builtin Agent`: Knovum's built-in collaboration product.
- `Agent Runtime`: the replaceable model-tool control mechanism used by Builtin Agent.
- `External Agent`: an ACP or equivalent external controller, not an AI provider.
- `Authorization`: principal, scope, grant and approval; separate from content review.

Names such as `manager`, `service`, `helper`, `utils`, `harness` and generic `engine` are not default
owners. A module name must identify the policy or aggregate it owns.

## Dependency direction

```text
Vue typed commands
        |
        v
Space Builtin Agent    Workflow runtime    Space MCP
        \               |               /
         +---------- SpaceAction -------+
                       |
          domain owners + Authorization + Runs
                       |
              Resource revisions and provenance
```

- `ai` owns model/media contracts, providers, connection resolution and transport. It does not own
  external Agent sessions or Space policy.
- `agent_runtime` depends on vendor-neutral model-turn and tool contracts, not on Spaces.
- `external_agents` owns discovery, connection, ACP sessions and protocol events; it does not depend
  on Spaces.
- `spaces/actions` owns definitions, discovery and invocation coordination. Concrete behavior remains
  with nodes, knowledge, assets, structure, derivation and other domain owners.
- `spaces/runs` is the durable execution authority. Process-local registries remain implementation
  details of their owning engines.
- `spaces/workflows` owns definitions, compilation and execution semantics. It may invoke
  SpaceActions, but does not depend on Builtin Agent implementation.
- `spaces/mcp` is an inbound protocol projection and cannot mutate SQLite directly.
- Vue presentation consumes typed capability clients and query state; it never owns durable Run or
  Workflow execution truth.

## Identity and version boundaries

Content revisions, canvas revisions, Run state versions and published Workflow versions are distinct.
An input reference includes the resource revision and any page, block, time, region or table locator
required to reproduce the actual read.

Semantic relations, production derivations and Workflow execution edges are three different graphs.
None may be inferred from another without an explicit domain rule.

Generated identity, source freshness, human review, resource availability and Run state are separate
facts. A single mixed “trusted” status is forbidden.

## SpaceAction contract

A SpaceAction definition includes:

- stable ID and semantic version;
- typed input and output ports;
- parameter schema;
- read, write, external-transfer and cost effects;
- authorization and approval requirements;
- executor capability requirements;
- input consistency policy;
- retry, reuse and publication policy;
- classified errors.

The WebView does not receive a universal `invoke_action(actionId, json)` command. User-facing IPC
remains intent-specific and calls the same Rust owner used by Workflow, Builtin Agent and MCP.

## Durable execution

Runs persist their trusted initiator, scope, frozen definition and dependency versions, input
snapshots, executor bindings, budget, authorization references, publication plan and state sequence.
StepInstances persist resolved inputs and outputs. Attempts persist the concrete request, executor,
lease generation, external task identity, usage, checkpoint and outcome.

Model calls, network work and human waiting never hold a database write transaction. Publication uses
managed staging followed by a short transaction that rechecks authorization, cancellation, execution
generation and expected resource revision.

Late workers and uncertain external results cannot publish. Retry creates a new Attempt; rerunning a
terminal Run creates a related new Run.

Batch expansion has a durable item identity `(retry lineage, definition node, item key)`. Successful
items and active child Runs are reused across selective retry, while failed items receive a new
attempt. Expanded-step, attempt, token, byte, duration and parallelism budgets apply to the Run tree.

## Workflow

A Workflow is a typed immutable published definition. Draft validity, publishability, environment
readiness and authorization are evaluated separately. Graph and outline editors mutate the same draft
command model.

Common graph algorithms use `petgraph`; pure conditions and mappings use bounded CEL; schedules use
`cron` with `chrono-tz`. These libraries do not own Knovum publication, authorization, retry, DST,
misfire or recovery policy.

## Builtin Agent and external collaboration

Builtin Agent exposes explicit Ask, Plan and Execute modes. Mode changes never enlarge scope. Tool calls
are persisted before side effects execute, and committed results are represented by Receipts rather
than model prose.

The Builtin Agent's general model-tool loop is isolated behind `agent_runtime`. Model IO remains on the
single `genai`-backed path. External Agents stay in `external_agents`; failure never silently switches
between built-in and external controllers.

Builtin Agent tasks persist explicit model-turn, tool-call, token and duration limits. Long conversation
context uses a bounded recent transcript plus traceable extractive segments; deletion removes those
segments and control checkpoints. External artifact sessions are durable external-Agent Runs whose
user-visible messages, tool state and permission requests are observable without persisting thoughts.

## Migration rule

Capabilities migrate owner by owner. A migration changes the owner and contract, switches every
caller, migrates data if required, updates tests and documentation, then deletes the old path. Runtime
fallbacks, long-lived dual writes, command aliases and fabricated historical facts are forbidden.

## Quality gates

- TypeScript and Vue source files stay below Oxlint's 180-effective-line ceiling, and their
  complexity stays at or below 10. These limits are ceilings, not targets.
- Rust structure is reviewed by responsibility and verified by rustfmt, Clippy and tests rather than
  repository-specific line or parameter counters.
- Oxc, dependency-cruiser, Knip, jscpd, rustfmt, Clippy and tests remain enabled.
- New dependencies require exact role, maintenance, license, compatibility, security, packaging and
  exit-strategy evidence in `docs/oss-first.md`.
- Tests cover contracts, SQLite transactions, graph compilation, execution recovery, authorization,
  protocol projections, asset lifecycle, accessibility and desktop end-to-end behavior.
- Unauthorized writes, lost edits, duplicate irreversible effects, post-cancel publication, false
  success, false evidence, secret leakage and unrecoverable migration damage block release.
