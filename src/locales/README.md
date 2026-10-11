# Locale organization

`locale-registry.ts` is the only registry. It eagerly imports `./*/**/*.json` and derives
namespaces from paths: `spaces/media/image-editor.json` owns `spaces.media.imageEditor.*`.
Language root directories use the recommended [BCP 47 casing](https://www.rfc-editor.org/rfc/rfc5646.html#section-2.1.1)
and match application locale identifiers: language lowercase (`en`), script title case (`Hans`),
and alphabetic region uppercase (`CN`), for example `zh-CN` or `zh-Hans-CN`. A region is optional;
`en` identifies general English. Module paths below each language root use kebab-case; object keys
use lowerCamelCase. Each language has the same module tree.

The display languages are `en`, `zh-CN`, `zh-TW`, `ja`, `ko`, `fr`, `de`, `es`, `pt-BR`, `it`, and `ru`.
Portuguese uses Brazilian wording. Arabic is not a display language. The AI translation target
catalog is a separate capability and may include languages unavailable in the interface.

## Module boundaries

- `common.json` owns neutral actions, generic fields and shared interface states. A message is
  common because its meaning is shared, not because its spelling matches another message.
  Permission scope, destructive consequences and feature-specific recovery stay with their owner.
  Shared search empty states and named enable/disable actions belong here, rather than under one
  Extensions subtype. Connector-definition and connection-instance discard confirmations remain
  separate under `extensions.connectors.discard` and `extensions.connections.discard`.
- `ai`, `errors`, `tasks`, `updates` and `dictation` own foundation capabilities; `shell` owns workbench and
  startup wording. These are domain packages, not extensions of `common`.
- `settings/*` owns settings and navigation groups. `spaces/*` owns Space behavior.
- `settings/ai/` separates the settings overview, provider connections, model definitions, shared
  editor confirmations and local models. `settings.credentials` owns API-key controls shared by
  settings panes. Provider connection partial-save failures remain under `settings.ai.connections`.
  `settings.archivedSpaces` owns the archived-Spaces settings view; `settings.general` owns settings
  navigation, loading/reset behavior and shared settings-save errors.
- `settings.appearance.options.language` owns the display-language choices, including Brazilian
  Portuguese. `ai.languages` owns AI translation targets, including general Portuguese and Arabic.
  Keep these catalogs separate even where language names currently match.
  `ai.localModels.unavailable` owns shared model-availability outcomes displayed by both model
  settings and speech model selection; the settings namespace owns model management controls.
- `updates` owns updater actions, progress, failures and restart wording across settings, application
  notifications and the Task Center. `errors.windowClose` owns failed close/save outcomes.
- `spaces.media.transform` owns transformations shared by image and video editing. Editor-specific
  operations stay under their editor; `spaces.media.imageComments` owns image annotations and their
  conversation context. Shared media playback failures belong to `spaces.media.player`.
  `extensions.plugins.source` owns plugin package selection; `extensions.catalog` owns MCP resources,
  templates and prompts. `extensions.endpoint` owns endpoint fields shared by connector definitions
  and connection instances. Declared plugin commands and connector-specific configuration or conflict
  messages remain under `extensions.connectors`.
- `spaces/creation/shared.json` owns messages shared by creation types, such as presets, source
  citations and derived resources. Other Space capabilities retain their own messages.
- `extensions/*` separates overview, plugins, skills, connector definitions, connection instances,
  accounts and the resource catalog. Conversation approval, uncertain outcomes and interactions
  belong to `spaces/conversation.json`, not Extensions management.
- `spaces/speech/*` separates the library, voices, reference audio, projects, audiobooks, video
  dubbing, separation, translation, signal inspection and model selection. Keep one responsibility
  per module; split a growing module at a real domain boundary.
  `spaces.speech.preview` owns voice/project audition controls and sample-text fields; library voice
  selection belongs to `spaces.speech.library`, while candidate generation stays with the voice.
- A feature normally uses its own namespace and `common.*`, plus foundation capabilities it
  actually consumes. Spaces may share the Extensions catalog actions
  `extensions.catalog.{browse,parameters,read}` and the connector noun
  `extensions.overview.mcp`. Settings navigation may reference destination titles and search
  terms in its registry. These metadata references do not move business messages into `common`.
  The archived-Spaces view uses `spaces.space.inbox` as the canonical Inbox name through its archive
  contract. Shared endpoint fields describe the same endpoint contract in connector and connection
  editors. Prefer these named capability contracts over borrowing unrelated screen wording.
- A parent JSON must not coexist with a same-named directory. Delete `x.json` when splitting it
  into `x/`. Static structure tests reject parent/child and normalized path collisions; the runtime
  registry alone does not detect every conflicting assignment.

## Key naming and reuse

Name the semantic role, grouped within its owner: `archived.delete.{title,description,failed}`.
Avoid new flattened families such as `deleteArchivedSpaceDescription`, screen-position names,
ambiguous abbreviations and aliases for retired keys. Serialized domain-value maps are the only
explicit naming exemptions.

Group related roles as `discard.{title,description}`, `confirm.{title,description}` and
`prompt.{label,placeholder}`. Keys describe the operation or state, rather than the current widget
position. A single message may keep a descriptive key such as `saveFailed`; do not create a one-key
group just to shorten its name. Retired keys are removed from every language and dynamic declaration.

Reuse neutral actions through `common`, retaining the target object and consequences in nearby
labels or confirmations. Colors, domain statuses, parameters and nouns may have separate keys when
used in different contracts. Do not deduplicate messages solely by their current rendered value.

## Desktop wording

Use short action labels and state messages. Empty states should identify what is missing and, when
needed, the next action. Avoid tutorials, implementation descriptions and repeating information
already evident from a control. Keep constraints that affect a choice, permission scope, permanent
deletion, uncertain external outcomes and actionable error recovery explicit.

Translate a complete sentence, including counts and units. Count messages use vue-i18n plural forms;
pass named `{ count }` and an explicit plural count when needed. Chinese, Japanese, and Korean can use
one form. Russian uses `one | few | many` forms selected through `Intl.PluralRules`; French and Brazilian
Portuguese use the singular form for zero and one. All languages expose the same unique placeholder names.

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
value map. Do not exempt a broad namespace to silence one missing reference. Update all languages,
all consumers and declarations together; do not introduce compatibility aliases.

Local messages remain eagerly loaded so desktop startup and controls do not depend on asynchronous
language requests. Adding a business module needs only matching files under every language root.
Adding another language requires updating the supported locale contract and checks as well as its
complete module tree. Introduce loading by capability only if measured bundle costs justify it.
