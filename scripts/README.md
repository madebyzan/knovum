# Repository tools

Daily development uses five commands: `pnpm desktop:dev`, `pnpm test`, `pnpm format`,
`pnpm check` and `pnpm package`. Use the vendor CLI's flags for watch, fix and bundle variants.
`dev` and `build` are Tauri's Vite hooks. `desktop:dev` prepares FFmpeg and native libraries,
builds the worker with Cargo, then starts Tauri; it does not prepare Python.

These are auxiliary tools for one desktop product. Prefer package scripts and existing vendor
CLIs. Retain custom code for concrete resource, contract, verification or publication policy;
do not build a task runner, installer framework or a second dependency manager. Avoid extra
public commands and handwritten duplicate schemas. Test fixtures are not developer entry points.

## Occasional operations

- `contracts:generate` / `contracts:check`: Rust-owned TS/Agent contracts and drift checks.
- `e2e`: the isolated desktop test flavor.
- `runtime:prepare`, `runtime:build`, `runtime:provision`: the retained Agent and controlled
  resource supply. These serve explicit resource preparation and release workflows. The installed
  product consumes finished resources and never invokes repository producers or uv/pip.

## CI and ownership

| Workflow                | Responsibility                                                                                     |
| ----------------------- | -------------------------------------------------------------------------------------------------- |
| `quality.yml`           | Source/unit/audit gates, all three desktop compilation targets and actual signing rejection tests. |
| `python-runtime.yml`    | Explicit Agent and FFmpeg publication on their target hosts.                                       |
| `release-candidate.yml` | Shared QA/candidate build preparation; only protected tag releases sign and publish candidates.    |
| `promote-update.yml`    | Explicit promotion of a verified candidate.                                                        |

`desktop/` owns desktop preflight and resource assembly; `runtime/` owns controlled resource
production and verification; `release/` owns distribution policy; `checks/` contains only
capability policy, contract drift and the retained two-environment Python checks.
Each owner's `__tests__/` contains its regressions.
`pnpm check:scripts` discovers Node tests; `check:python` covers retained Python tooling.

Model qualification uses actual engine tests and reviewed measurements. There is no separate
repository scoring CLI, corpus schema or synthetic qualification fixture collection. Naming and
directory conventions stay in review; do not add custom scanners that duplicate language tooling.

Exact supply, signing and platform obligations belong to [packaging](../docs/packaging.md)
and [release](../docs/release.md), rather than a separate tooling contract.
