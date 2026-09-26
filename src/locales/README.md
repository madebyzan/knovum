# Locale organization

`locale-registry.ts` is the only registry. It eager-globs `./{en,zh-cn}/**\/*.json` and derives each
message namespace from the file path: path segments are kebab-case on disk and lowerCamelCase in
code, so `spaces/media/image-editor.json` owns `spaces.media.imageEditor.*`. There is no manual
composition step and no second source of truth.

Each locale is a self-contained directory with an identical module tree. A locale directory uses the
lowercase form of its BCP-47 tag (`zh-cn`); the registry maps it to the canonical tag (`zh-CN`) that
the rest of the application uses as the locale identifier.

## Module boundaries

- One JSON file owns one product area. A file and a directory of the same name must never coexist:
  the registry resolves paths in glob order and throws `Locale path conflict` when a namespace is
  claimed twice. Splitting `x.json` into `x/` means deleting `x.json` in the same change.
- Top-level namespaces are `ai`, `common`, `errors`, `shell`, `tasks`, plus the `settings.*` and
  `spaces.*` trees. `common.json` holds shared interface language (confirm/cancel/save, generic
  field labels); everything else stays with the feature that owns the behavior.
- A feature translates only its own namespace and `common.*`. Settings panes do not reach into
  `spaces.*`, and Spaces does not reach into `settings.*` — when both surfaces need the same wording,
  each owns a key, or the wording belongs in `common.*`.

## Key naming

- Keys use descriptive `lowerCamelCase` and name meaning, not screen position.
- Group by role, not by flattening: `archived.delete.label` / `.title` / `.description` / `.failed`
  rather than `deleteArchivedSpace`, `deleteArchivedSpaceTitle`, `deleteArchivedSpaceDescription`,
  `deleteArchivedSpaceFailed`.
- A group name is singular for a collection owned by one entity (`streak.current`, `streak.longest`)
  and plural for an enumerated set (`weekdays`, `categories`, `presets`).
- Do not add aliases, compatibility keys, duplicated labels, or runtime fallback messages. Reject or
  reset obsolete pre-release data instead of reinterpreting it.

## Wording

Simplified Chinese follows one term per concept:

| Concept              | Term     | Not             |
| -------------------- | -------- | --------------- |
| credential           | 凭据     | 凭证            |
| conversation         | 对话     | 会话            |
| workflow             | 工作流   | 流程            |
| visual understanding | 视觉理解 | 视觉分析        |
| space portal         | 空间入口 | 传送门 / Portal |
| remove (reversible)  | 移除     | 删除            |
| delete (destructive) | 删除     | 移除            |
| document/data text   | 文本     | 文字            |
| visual text object   | 文字     | 文本            |
| group (noun)         | 组合     | 分组            |
| group (verb)         | 编组     | 组合            |

English uses sentence case. Only the first word, proper nouns, and product/domain terms stay
capitalized: Knovum, Space, Agent, Portal, Task Center, Markdown, JSON Canvas, Office, Word, PDF,
Qwen, Whisper, Rust, Node.js, key names (Enter, Shift, Option, Alt), and acronyms. `·` and `|`
separate independent items, so the word after them keeps its capital.

## Gates

`src/foundation/i18n/__tests__/locale-completeness.test.ts` enforces the contract:

1. English and Simplified Chinese key trees stay aligned.
2. Every statically referenced key exists.
3. Placeholders `{name}` match between languages.
4. Module paths match between languages.
5. Message keys are lowerCamelCase. Namespaces keyed by a serialized domain value are exempt and
   listed in `serializedValueMaps`.
6. Every key is reachable from product code — either named by a dotted literal, or listed in
   `dynamic-keys.ts` for keys resolved at runtime from a domain value.
7. That allowlist stays narrowed to keys that still exist, so a stale entry fails the build.

When a key is resolved through a variable (`labelKey`, a stage name, a serialized enum), add the
narrowest possible prefix or exact key to `dynamic-locale-keys.ts`. Never exempt a whole large
namespace to silence one dynamic lookup.
