# Document conversion

This document owns document conversion routes, their supported subsets and renderer verification criteria.
Space persistence, revision validation, and lineage remain owned by `spaces.md` and `knowledge.md`.

## Implementation boundary

`src-tauri/src/documents/generation` is a conversion engine. It does not read a Space database,
authorize paths, download remote content or persist a second document model. Spaces supplies either
a revision-checked Markdown `DocumentSource` containing title/body/literal citations, or a Rust-authorized
Space-owned absolute file path with its SHA-256 revision. The private worker reads one bounded byte
snapshot and verifies its digest before parsing. The host checks the file again before publication.
`pulldown-cmark` parses Markdown into a temporary semantic document.
The DOCX, PDF and TXT backends consume that same model. HTML uses that validation and title policy,
then passes Markdown events to the pinned `pulldown-cmark` HTML writer for serialization.

The local `convert-document-to-docx`, `convert-document-to-pdf`, `convert-document-to-txt`, `convert-document-to-html` and `render-presentation` Actions share `spaces/derivation/document_formats` and `document_publication`. Canvas, Builtin Agent/MCP and saved workflow steps use the same Action Run. Rust reserves the managed file node, pins one current resource, converts in the application’s private document worker process and publishes with cancellation/epoch/source checks. The Run attempt cancellation token is the sole cancellation authority; the host terminates and waits for the worker before returning and releasing execution capacity. Failed or stale output bytes are removed. Vue submits Action intent and source identity; it neither allocates paths nor serializes bytes. This is format conversion with `derives-from` lineage. Download saves the existing resource; it is separate from conversion.

## Input routes

| Source             | Supported targets (excluding the current format) | Content policy                                                                                                            |
| ------------------ | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Markdown           | DOCX, PDF, TXT, HTML, PPTX                       | Existing Markdown subset below                                                                                            |
| Text-bearing PDF   | DOCX, TXT, HTML, PPTX                            | `pdf-inspector` extracts structural Markdown; content is reflowed without original images or page geometry                |
| DOC/DOCX, XLS/XLSX | DOCX, PDF, TXT, HTML                             | `office_oxide` reads the source; DOCX/HTML/TXT use its IR writers, PDF uses its Markdown writer and the existing renderer |
| PPT/PPTX           | DOCX, PDF, TXT, HTML; PPT also supports PPTX     | DOCX/HTML and legacy PPT upgrade use the vendor IR; PPTX preserves supported presentation content                         |

One current source is required; processing, failed and unsupported resources are rejected. Same-format
conversion is rejected by Rust as well as hidden in the menu. Cached `previewText`/`extractedText` is
never the conversion source. A PDF page requiring OCR fails the whole conversion rather than losing
that page; an empty PDF text result fails rather than generating a title-only file. Conversion does
not run AI recognition or switch to another engine after failure.

File display titles name the derived resource and HTML document metadata. They are not inserted as
new body headings in DOCX/PDF/TXT/HTML conversion. Text presentation conversion may use the display
title for its cover slide. Original document headings remain content.

Office conversion preserves only the selected library's supported content, not full Office layout or
animations. Vendor IR writing retains supported tables, text styling and embedded images in DOCX;
standalone HTML retains tables and PNG/JPEG/GIF/BMP images as data URLs. Remote images are never
fetched. PDF conversion rejects image-bearing Office content. Unsupported image formats or missing
embedded bytes fail in image-preserving targets. Vector drawings and hidden sections fail explicitly.
DOCX conversion rejects presentation speaker notes; legacy PPT upgrade retains notes. Word/Excel to
PPTX is not exposed because the library's section-to-slide mapping is not a document pagination engine.
Spreadsheets convert displayed cell content; formulas, workbook behavior and original sheet layout are
not preserved in these non-spreadsheet targets.

Office admission reuses the extraction engine's ZIP limits: 128 MiB source, 20,000 entries, 128 MiB per
expanded entry and 256 MiB total expanded content. Conversion bounds semantic text to 4 MiB, sections
to 2,000, nesting to 64, elements to 200,000, table columns to 64 and embedded image bytes to 64 MiB.
Legacy PPT upgrade is capped at 100 slides. Vendor writer truncation is an error.

## Markdown renderers

CommonMark paragraphs, H1–H6 headings, quotes, thematic breaks, soft/hard line breaks, links,
inline code, fenced/indented code, and nested ordered/unordered lists are represented explicitly.
Only tables, strikethrough, and task lists are enabled as parser extensions. Parser upgrades must
not expand this subset implicitly through `Options::all()`.

DOCX writes OOXML table cells, numbering definitions and hyperlink relationships, with declared
Normal, Title, Heading1–Heading6, and code styles. PDF emits structural Typst functions using a
checked-in `pdf/document.typ` profile. Literal text is always a quoted function argument, never raw
Typst markup. Conversion has no remote-content or filesystem/package resolver.

Markdown TXT intentionally loses visual formatting. It retains heading text, list order and nesting, checkbox
state, code, readable link destinations, tab-separated table cells, and image alternative text.
It never includes an image's private filesystem path. A title-only document is valid. A leading H1
is removed only when its plain text equals the separately stored title; distinct body headings remain.
Citations are appended as literal structured source paragraphs, not interpolated Markdown.

PPTX uses the already pinned `office_oxide` writer. Knovum maps title and Markdown sections into at most 100 plain-text slides; the library owns package XML, relationships and escaping. It does not promise direct Markdown table/list styling, images, templates, animations or pixel fidelity. Existing shared Markdown validation applies before serialization.

Markdown HTML emits a standalone UTF-8 document through `pulldown-cmark::html::push_html`, including headings, lists,
tables, code, task state and literal citations. Its fixed stylesheet supports screen and print output.
The library owns HTML tags, text escaping and link-attribute escaping; Knovum supplies the static document shell and literal title/citation events. Only HTTP, HTTPS and mailto links are active;
relative and private-file links fail. A restrictive Content Security Policy disables scripts, external
resources, base URLs and form submission; only embedded data images are allowed for Office HTML. Image-containing Markdown HTML conversions fail explicitly: the engine
does not read image files, fetch URLs or silently omit them.

## Explicit limitations

Markdown conversion does **not** embed images in PDF/DOCX/HTML, render math or footnotes,
provide custom templates, or export rich Tiptap notes. Image-containing Markdown DOCX/PDF/HTML conversions fail
explicitly instead of silently dropping images. TXT and the plain-text PPTX subset represent images by their alternative text.
Raw HTML is rejected in every backend rather than stripped. Disabled Markdown extensions are not
promised semantic rendering. Active DOCX/PDF/HTML hyperlinks currently accept only HTTP, HTTPS, and mailto;
relative links, heading anchors, file links, and executable schemes fail explicitly.

PDF still uses platform-installed fonts. It is not a bundled-font or cross-platform pixel-identity
guarantee. Nonfatal Typst warnings log their count without logging source text. Compilation/export
errors remain failures.
Typst compilation remains synchronous inside the worker. Process isolation supplies interruption without changing the renderer or letting abandoned background threads keep computing.

Input is bounded to 4 MiB across title/body/citations, 4,096 citations, 200,000 parser events,
64 parser nesting frames, and 64 table columns. Unsupported control/XML characters are rejected.
The private protocol limits its JSON request to 16 MiB (including encoding overhead), its response header to 16 KiB, and output to 64 MiB. Unknown fields, wrong versions, malformed/trailing frames and non-zero worker exits fail explicitly. There is no in-process fallback. Run budgets and cancellation govern execution; parser limits alone are not a hard CPU-time sandbox.

## Verification before merge

Focused checks cover the semantic model, title/citation policy, safe output, unsupported inputs,
DOCX package XML, revision identity and shared Action execution. The platform PDF smoke requires a
platform CJK font.

Run on the supported desktop development environment:

```sh
pnpm exec vitest run src/features/spaces/__tests__/document-conversion.test.ts src/features/spaces/__tests__/document-conversion-catalog.test.ts
cargo test --locked --manifest-path src-tauri/Cargo.toml documents::generation
cargo test --locked --manifest-path src-tauri/Cargo.toml spaces::derivation::tests::document_formats
```

macOS and Windows build checks and opening representative output in Word/PDF viewers are
required before treating this as release-ready. Binary signatures and XML assertions do not prove
visual fidelity, pagination quality, or full application lifecycle correctness.

Dependency selection and format-translation boundaries are recorded in [oss-first.md](oss-first.md).
