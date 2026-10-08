# Dependency policy

This document governs how Knovum admits, pins, audits, upgrades and removes dependencies. Exact
versions belong to the committed npm, Cargo and Python lockfiles; capability behavior belongs to
its owning architecture or product document.

## Admission

A direct dependency must have a current production or tooling caller, satisfy a documented peer or
runtime requirement, or implement a capability selected through [oss-first.md](oss-first.md).
Before adding one:

1. Identify the capability owner and the exact requirement the dependency satisfies.
2. Prefer an existing dependency or an official Tauri, Vue or platform API when it fits.
3. Review maintenance, security, license, target support and transitive cost.
4. Add only the features and target scopes used by the product.
5. Record why credible alternatives fail a named requirement.

Do not add speculative SDKs, duplicate engines, rename-only wrappers or packages retained only for
future use. Import vendor APIs directly unless Knovum adds policy, validation, security, domain
translation or multi-step orchestration. Knip exceptions are limited to dependencies or generated
artifacts that static analysis cannot observe.

Tauri plugins are admitted with the product flow that calls them. Installation does not grant the
WebView permission: capabilities remain narrowly scoped, SQL stays Rust-owned, and native platform
bindings remain target dependencies behind `cfg(target_os)`. Desktop behavior and permission
ownership are defined in [desktop-foundation.md](desktop-foundation.md) and
[system-architecture.md](system-architecture.md).

## Pinning and lock ownership

- `pnpm-lock.yaml`, `src-tauri/Cargo.lock` and each Python runtime's `uv.lock` are generated only by
  their owning package managers and are committed.
- Exact pins are required where an SDK protocol, native ABI, model/runtime supplier or coordinated
  package family can change product behavior. Compatible ranges are acceptable only where the
  lockfile remains the shipped identity and the public API is stable.
- Coordinated families such as Tauri core/API/CLI/plugins, Tiptap packages, WebdriverIO adapters and
  native Python stacks are upgraded as a tested unit. A single-package version bump is not a
  completed migration when public types, peer constraints or native ABI cross package boundaries.
- Transitive features are not relied on for production callers. If Knovum imports an API or requires
  a feature, declare the dependency or feature directly.
- Registry mirrors, metadata rewrites, advisory suppression and local vendor edits are not used to
  force an otherwise incompatible dependency into the graph.

The application toolchain uses Oxfmt, Oxlint, `vue-tsc`, dependency-cruiser, Knip and jscpd for
separate concerns. Do not add a parallel formatter or linter stack. Type-aware linting may replace
`vue-tsc` only after the stable TypeScript and Oxc toolchains are mutually compatible and the same
strict checks pass.

## Upgrade and removal

An upgrade must regenerate the owning lockfile, preserve the declared security floor and license
policy, and pass the focused contracts plus the native checks required by the affected target.
Availability in a registry or successful resolution does not establish ABI, model or platform
compatibility.

Remove a dependency when its last caller is removed, an existing maintained dependency takes over
its responsibility, or it cannot satisfy the security, license or platform contract. Remove its
features, permissions, build inputs, exceptions and generated evidence in the same change. Do not
retain obsolete installers, aliases, compatibility readers or dormant runtime profiles.

Dependency families blocked by an upstream defect or public-type incompatibility stay pinned until
a published compatible release exists. The upgrade change must reproduce the original failing case;
a higher version number alone is insufficient.

## Security gates

CI installs from committed lockfiles and runs the repository's production npm audit/signature
checks and Cargo advisory/license policy. `deny.toml` is the Rust license allowlist; changes require
the same accountable release/legal review as the dependency they admit. Dependabot is an update
signal, not evidence that an update is compatible or approved.

Security floors recorded in runtime manifests must not be lowered. This includes the selected
PyTorch line's fix for
[`GHSA-rrmf-rvhw-rf47`](https://github.com/advisories/GHSA-rrmf-rvhw-rf47), the MLX
unsafe-deserialization floor, and the contract generator's schema/template, local-file and SSRF
fixes. Exact current versions remain lockfile-owned.

Temporary advisory acceptance must identify the advisory and dependency path, exposure review,
accountable owner, review date and removal condition. Opening an issue or passing unrelated tests is
not acceptance. Broad suppressions, misleading supplier metadata and feature changes made only to
hide an advisory are prohibited.

### Current temporary dependency exceptions

- `citationberg` is patched to the immutable official upstream revision
  `06a591e2f237d25e1dfdedac3f3d1494c496c52d` from
  [upstream PR #44](https://github.com/typst/citationberg/pull/44), which moves its XML dependency
  off the vulnerable release. Remove the patch when a compatible published `citationberg` release
  contains that change; regenerate `src-tauri/Cargo.lock`, confirm the vulnerable XML versions are
  absent, and run the affected document-export and native checks.
- `tauri-plugin-clipboard-api@2.1.11 > valibot` is overridden to `1.4.2`. Remove the override when a
  compatible plugin release selects a supported secure Valibot version and the valid and malformed
  clipboard payload regressions pass.

No exception constitutes platform, distribution or legal approval.

## Native and Python supply chain

The application-managed CPython runtimes, native media closure and native inference components are
producer outputs, not development-environment copies. Their manifests and per-target locks own the
interpreter, packages, native libraries, source archives, hashes and supplier identities. End-user
machines never resolve or install production dependencies.

Every conveyed target inventory must be bound to:

- exact installed files and cryptographic identities;
- package and native-library license notices;
- corresponding source and build inputs where required;
- a generated SBOM for the actual published inventory;
- relocation and native execution evidence for that target.

Package metadata, a top-level repository license, populated hashes or execution on another target do
not approve distribution. GPL/LGPL and other source-offer obligations apply to the complete conveyed
native closure, including FFmpeg, eSpeak NG, piper and any libraries embedded by a wheel or archive.
The compiled-source evidence and license-review records remain required packaging inputs.

Each `python/runtimes/<profile>/dependency-license-review.json` starts unapproved. Production
packaging may consume it only after an accountable reviewer binds the frozen requirements/lock
identity to each target's exact dependency evidence. A changed dependency, source revision or
installed inventory invalidates that review. QA results cannot substitute for legal/license
approval.

Model files are independently downloaded resources. Their source, license, hash and runtime
compatibility are reviewed independently from both the desktop installer and Python runtime. Model
selection and acquisition contracts live in [local-models.md](local-models.md); publication and
platform gates live in [packaging.md](packaging.md) and [release.md](release.md).

## Verification ownership

Dependency changes run the smallest checks that prove the affected boundary, followed by the
repository lint/type/dependency gates. Native runtime, desktop lifecycle and release changes also
require supported-target evidence; a local compile does not claim that evidence.

Keep verification output with CI or release artifacts. This policy records durable rules and live
exceptions, not dated test logs, machine-local probe results or lockfile inventories.
