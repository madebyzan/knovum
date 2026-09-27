## Change

Describe the user/product behavior changed.

## Architecture gate

- [ ] The change respects the enforced graph: composition root/app may compose modules; shell and features may depend on foundation/shared; foundation may depend on shared; shared is a leaf; features remain isolated.
- [ ] There is one source of truth; no duplicate registry/state/config was introduced.
- [ ] No rename-only wrapper, universal service, giant composable, or catch-all utility was introduced.
- [ ] Required behavior fails explicitly; no blanket compatibility/best-effort fallback was added.
- [ ] Responsibilities remain focused and testable; frontend code passes the existing Oxlint limits.
- [ ] Architecture/UI/runtime/packaging documentation was updated when a repository contract changed.

## OSS-first gate

For non-trivial new functionality or a new dependency:

**Capability / requirements:**

**GitHub + npm/crates.io candidates checked:**

**Selected and why:**

**Rejected candidates and exact unmet requirement:**

**Security / Tauri capability impact:**

**Why custom code is necessary (custom code only):**

## Verification

- [ ] `pnpm check:web`
- [ ] `pnpm check:rust`
- [ ] Manual desktop behavior checked on affected OS(es)
- [ ] Installer/package behavior checked when bundling, signing, updater or install semantics changed
