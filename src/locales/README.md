# Locale organization

Each locale is a self-contained directory with the same module tree. `messages.ts` composes that
tree, and `locale-registry.ts` is the only application-level registry. File names use kebab-case;
their message namespaces use the equivalent lowerCamelCase name.

## Rules

- A JSON file owns one product area. Related fragments may compose one public message namespace;
  only `messages.ts` performs that composition.
- A locale directory uses the lowercase form of its BCP-47 tag (`zh-cn`); `locale-registry.ts` maps
  it to the canonical tag (`zh-CN`) that the rest of the application uses as the locale identifier.
- Message keys use descriptive `lowerCamelCase` names and describe meaning, not screen position.
- Shared interface language belongs in `common.json`; feature-specific language stays with its owner.
- Serialized domain values may be used verbatim only inside an explicit lookup map such as
  `relation`, `aiScope`, `knowledgeHealthIssue`, or `enrichmentJobType`.
- Locale trees, placeholders, and module paths must stay aligned.
- Do not add aliases, compatibility keys, duplicated labels, or runtime fallback messages.
