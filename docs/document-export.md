# Markdown document export

This document owns the supported Markdown export subset and renderer verification criteria.
Space persistence, revision validation, and lineage remain owned by `spaces.md` and `knowledge.md`.

## Implementation boundary

`src-tauri/src/documents/generation` is an export-only engine. It does not read a Space database,
open source paths, download remote content, or persist a second document model. Spaces loads one
revision-checked `DocumentSource` containing the separately stored title, Markdown body, and literal
citation records. `pulldown-cmark` parses the body once per export into a temporary semantic document.
The DOCX, PDF, and TXT backends consume that same model.

The local `export-document-as-docx`, `export-document-as-pdf`, `export-document-as-txt` and `render-presentation` Actions share `spaces/derivation/document_formats` and `document_publication`. Canvas, Builtin Agent/MCP and saved workflow steps use the same Action Run. Rust reserves the managed file node, pins one current Markdown resource, renders off the async executor and publishes with cancellation/epoch/source checks. Failed or stale output bytes are removed. Vue submits Action intent and source identity; it neither allocates paths nor serializes bytes. This is format conversion with `derives-from` lineage. Ordinary Markdown `.md` download is unchanged.

## Supported subset

CommonMark paragraphs, H1–H6 headings, quotes, thematic breaks, soft/hard line breaks, links,
inline code, fenced/indented code, and nested ordered/unordered lists are represented explicitly.
Only tables, strikethrough, and task lists are enabled as parser extensions. Parser upgrades must
not expand this subset implicitly through `Options::all()`.

DOCX writes native table cells, numbering definitions and hyperlink relationships, with declared
Normal, Title, Heading1–Heading6, and code styles. PDF emits structural Typst functions using a
checked-in `pdf/document.typ` profile. Literal text is always a quoted function argument, never raw
Typst markup. Export has no remote-content or filesystem/package resolver.

TXT intentionally loses visual formatting. It retains heading text, list order and nesting, checkbox
state, code, readable link destinations, tab-separated table cells, and image alternative text.
It never includes an image's private filesystem path. A title-only document is valid. A leading H1
is removed only when its plain text equals the separately stored title; distinct body headings remain.
Citations are appended as literal structured source paragraphs, not interpolated Markdown.

## Explicit limitations

Export does **not** embed images in PDF/DOCX, render math or footnotes, export HTML,
provide custom templates, or export rich Tiptap notes. Image-containing binary exports now fail
explicitly instead of silently dropping images. TXT represents images by their alternative text.
Raw HTML is rejected in every backend rather than stripped. Disabled Markdown extensions are not
promised semantic rendering. Binary hyperlinks currently accept only HTTP, HTTPS, and mailto;
relative links, heading anchors, file links, and executable schemes fail explicitly.

PDF still uses platform-installed fonts. It is not a bundled-font or cross-platform pixel-identity
guarantee. Nonfatal Typst warnings log their count without logging source text. Compilation/export
errors remain failures.
The current synchronous Typst compilation stage does not introduce a new cancellation guarantee.

Input is bounded to 4 MiB across title/body/citations, 4,096 citations, 200,000 parser events,
64 parser nesting frames, and 64 table columns. Unsupported control/XML characters are rejected.
These limits bound conversion input; they are not a hard CPU-time or output-byte sandbox.

## Verification before merge

Focused checks cover the semantic model, title/citation policy, safe output, unsupported inputs,
DOCX package XML, revision identity and shared Action execution. The native PDF smoke requires a
platform CJK font.

Run on the supported desktop development environment:

```sh
pnpm test -- src/features/spaces/__tests__/document-export.test.ts
cargo test --locked --manifest-path src-tauri/Cargo.toml documents::generation
cargo test --locked --manifest-path src-tauri/Cargo.toml spaces::derivation::tests::document_formats
```

Native macOS and Windows build checks and opening representative output in Word/PDF viewers are
required before treating this as release-ready. Binary signatures and XML assertions do not prove
visual fidelity, pagination quality, or full application lifecycle correctness.

Dependency selection and format-translation boundaries are recorded in [oss-first.md](oss-first.md).
