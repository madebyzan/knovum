# Locale organization

`locale-registry.ts` is the only registry. It eagerly imports `./{en,zh-cn}/**/*.json` and derives
namespaces from paths: `spaces/media/image-editor.json` owns `spaces.media.imageEditor.*`.
Paths use kebab-case; object keys use lowerCamelCase. Each language has the same module tree.
Directory `zh-cn` maps to application locale `zh-CN`.

## Module boundaries

- `common.json` owns neutral actions, generic fields and shared interface states. A message is
  common because its meaning is shared, not because its spelling matches another message.
  Permission scope, destructive consequences and feature-specific recovery stay with their owner.
- `ai`, `errors`, `tasks` and `dictation` own foundation capabilities; `shell` owns workbench and
  startup wording. These are domain packages, not extensions of `common`.
- `settings/*` owns settings and navigation groups. `spaces/*` owns Space behavior.
- `spaces/creation/shared.json` owns messages shared by creation types, such as presets, source
  citations and derived resources. Other Space capabilities retain their own messages.
- `extensions/*` separates overview, plugins, skills, connector definitions, connection instances,
  accounts and the resource catalog. Conversation approval, uncertain outcomes and interactions
  belong to `spaces/conversation.json`, not Extensions management.
- `spaces/speech/*` separates the library, voices, reference audio, projects, audiobooks, video
  dubbing, separation, translation, signal inspection and model selection. Keep one responsibility
  per module; split a growing module at a real domain boundary.
- A feature normally uses its own namespace and `common.*`, plus foundation capabilities it
  actually consumes. Spaces may share the Extensions catalog actions
  `extensions.catalog.{browse,parameters,read}` and the connector noun
  `extensions.overview.mcp`. Settings navigation may reference destination titles and search
  terms in its registry. These metadata references do not move business messages into `common`.
- A parent JSON must not coexist with a same-named directory. Delete `x.json` when splitting it
  into `x/`. Static structure tests reject parent/child and normalized path collisions; the runtime
  registry alone does not detect every conflicting assignment.

## Key naming and reuse

Name the semantic role, grouped within its owner: `archived.delete.{title,description,failed}`.
Avoid new flattened families such as `deleteArchivedSpaceDescription`, screen-position names,
ambiguous abbreviations and aliases for retired keys. Serialized domain-value maps are the only
explicit naming exemptions.

Reuse neutral actions through `common`, retaining the target object and consequences in nearby
labels or confirmations. Colors, domain statuses, parameters and nouns may have separate keys when
used in different contracts. Do not deduplicate messages solely by their current rendered value.

## Desktop wording

Use short action labels and state messages. Empty states should identify what is missing and, when
needed, the next action. Avoid tutorials, implementation descriptions and repeating information
already evident from a control. Keep constraints that affect a choice, permission scope, permanent
deletion, uncertain external outcomes and actionable error recovery explicit.

Translate a complete sentence, including counts and units. English count messages use vue-i18n
plural forms; pass named `{ count }` and an explicit plural count when needed. Simplified Chinese
can use one form. Both languages must expose the same unique placeholder names.

English uses sentence case, retaining proper names, product terms and acronyms. Simplified Chinese
uses these terms consistently:

| Concept                  | Term        |
| ------------------------ | ----------- |
| Space                    | 空间        |
| conversation             | 对话        |
| preset                   | 预设        |
| voice profile            | 音色        |
| transcription            | 转录        |
| credential               | 凭据        |
| workflow                 | 工作流      |
| visual understanding     | 视觉理解    |
| space portal             | 空间入口    |
| remove from a collection | 移除        |
| destructive delete       | 删除        |
| document/data text       | 文本        |
| visual text object       | 文字        |
| group noun / verb        | 组合 / 编组 |

## Checks and updates

Run `pnpm exec vitest run src/foundation/i18n/__tests__` after locale changes. The checks cover:

1. Matching module and leaf-key trees across languages.
2. Existing literal references and matching unique placeholder names.
3. lowerCamelCase keys and kebab-case module paths.
4. Product reachability, excluding tests and locale infrastructure from the reference scan.
5. Narrow dynamic declarations that still resolve to existing keys.
6. Collision-free normalized paths, nonempty groups and nonblank string leaves.

Declare runtime-resolved keys in `dynamic-keys.ts`, preferring exact keys or a small serialized
value map. Do not exempt a broad namespace to silence one missing reference. Update both languages,
all consumers and declarations together; do not introduce compatibility aliases.

Local messages remain eagerly loaded so desktop startup and controls do not depend on asynchronous
language requests. Adding a business module needs only matching files under both language roots.
Adding another language requires updating the supported locale contract and checks as well as its
complete module tree. Introduce loading by capability only if measured bundle costs justify it.
