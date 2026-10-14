# Document conversion

This document owns conversion routes, content guarantees and renderer acceptance. Spaces owns
source authorization, revisions, persistence and lineage; packaging owns runtime distribution.

## Execution and ownership

Canvas, Agent/MCP and saved workflows invoke the same local Actions:
`convert-document-to-docx`, `convert-document-to-pdf`, `convert-document-to-txt`,
`convert-document-to-html` and `render-presentation`. Vue submits source identity and format intent.
Rust reserves the derived file node, validates the source revision, renders and atomically publishes
with `derives-from` lineage. Download saves an existing resource and is a separate operation.

The private document worker owns synchronous parsing and serialization. Markdown is parsed with
`pulldown-cmark`; Notes are translated directly from their validated Tiptap document into the admitted
`office_oxide` IR. PDF text is freshly extracted from the pinned original file, with revision-qualified
recognition supplied as a separate literal page input. Knovum keeps source/asset policy, format
translation, presentation pagination and safe typesetting.

Office original-layout PDF is outside this product boundary. Knovum does not install, bundle or
implicitly discover an Office layout engine. Users open the original file in their external Office
application, export PDF there and import the result. Office semantic content exports do not claim
original pagination or positioning. No second persistent document model, universal conversion
service, compatibility reader or alternate engine after failure is introduced.

The Run cancellation token owns cancellation. The host kills and reaps the worker
process tree before releasing execution capacity. Source files and attached image snapshots are
checked again before publication. Failed/stale output is removed. The private worker protocol is
strict v2; v1, unknown fields, wrong frame lengths, trailing bytes and unsuccessful exits fail.

## Supported routes

| Input              | Targets, excluding the current format   | Contract                                                                          |
| ------------------ | --------------------------------------- | --------------------------------------------------------------------------------- |
| Markdown           | DOCX, PDF, TXT, HTML, PPTX              | Semantic content and authorized managed images                                    |
| Rich Note          | DOCX, PDF, TXT, HTML, PPTX              | Structured authoring content; no Markdown intermediate                            |
| PDF                | DOCX, TXT, HTML, PPTX                   | Editable text reflow, source-page images; qualified recognition for scanned pages |
| DOC/DOCX, XLS/XLSX | DOCX, TXT, HTML                         | Supported semantic Office content; no original-layout PDF                         |
| PPT/PPTX           | DOCX, TXT, HTML; PPT also supports PPTX | Supported Office IR content and legacy upgrade                                    |

One published current source is required. Unsupported sources, processing/failed resources and
same-format conversion are rejected in Rust and excluded from the menu. Known scanned PDF pages
remain blocked until every required page has completed recognition with nonempty text from the
same source revision. Conversion never triggers an implicit AI request. The host validates the
recognition manifest; the worker validates required page identities against the original PDF.
OCR text is written literally, not interpreted as Markdown or executable content.

For text-bearing PDFs, original image regions are rendered with the existing native PDF adapter and
included as figures associated with their source page. Recognized scanned pages retain a raster of
the source page alongside editable recognized text. This is content delivery, not recovery of the
original editable Word layout. Page geometry and precise reading-order placement are not promised.
TXT carries semantic text and image alternative text without exporting private paths.

File display titles name the derived resource and HTML metadata. They do not become new body
headings during file conversion. Markdown and Note title/body handling avoids duplicating a matching
leading H1. Citations are appended as literal structured paragraphs.

## Markdown and Notes

Markdown supports CommonMark paragraphs, H1–H6 headings, quotes, rules, line breaks, links, inline
and block code, nested ordered/unordered lists, tables, strikethrough and task lists. Parser upgrades
must not expand the contract implicitly through `Options::all()`. Raw HTML is rejected. Math and
footnote extensions remain outside the Markdown authoring contract; custom templates are not an
export parameter.

Rich Notes retain headings, aligned paragraphs, lists and checked task state, code blocks, quotes,
rules, tables including merged cells in DOCX/PDF/HTML, and images. Supported text marks include
bold, italic, strike, underline, code, subscript/superscript, highlight and the editor's text styles.
Unsupported meaningful nodes/marks fail explicitly. ReasoningTrace is editor decoration state and
is not part of the saved authoring document. Structured data is translated directly to vendor IR.

Image resources are authorized by Spaces and revision-pinned before rendering. Markdown image
URLs may resolve through their stored editor asset identity, or an absolute/relative path within the
managed Space asset directory. Remote images are not downloaded; external/private file access is
not authorized by conversion. A missing image fails rich output rather than disappearing. TXT uses
alternative text and does not require image bytes. Workers read bounded snapshots, decode under
pixel/allocation limits, apply camera orientation metadata and normalize supported raster inputs to PNG.
DOCX applies one shared image-size policy across Markdown, Note, PDF and Office content routes:
retain aspect ratio, fit within the page body and preserve positive dimensions without upscaling.

DOCX uses the admitted writers for real OOXML tables, numbering, runs and hyperlinks. Markdown's
dedicated writer also preserves linked images; the Office/Note IR writer lacks that upstream
capability. These are fixed source routes, never alternate engines after failure. Note quotes in
DOCX are indented flowing paragraphs rather than positioned text boxes, so long quotes paginate.
Markdown
HTML uses the library event writer; Note/Office HTML uses the vendor IR writer. Both emit standalone
UTF-8 documents with the fixed screen/print stylesheet and restrictive CSP. HTTP, HTTPS, mailto,
relative and fragment hyperlinks are active. Scripts, forms, external resources and base URLs are disabled;
embedded data images are allowed. Absolute filesystem, private-file and executable hyperlinks fail.

Content PDF uses the checked-in Typst profile and in-memory image resolver. Text is always a quoted
function argument. No filesystem or package resolver is installed. Notes go directly from rich IR
to Typst, retaining text marks, alignment, tables and images. Platform-installed fonts remain the
font authority; cross-platform pixel identity is not guaranteed.

## Presentations

Markdown/Note/PDF presentations are content-derived decks, not Office layout replicas. The admitted
`office_oxide` writer owns package XML and relationships. Knovum owns a bounded slide layout:
16:9 pages, readable body font, Unicode grapheme-safe splitting, CJK-aware line estimates, a
100-slide cap, and title/paragraph/table/image placement. Long paragraphs and table cells split across
slides; table headers repeat for ordinary row pagination. Real tables, styled text runs, hyperlinks
and images are serialized. Images fit inside the slide body and keep alternative text.

Titles are capped at 32 CJK width units with overflow retained as body text. Presentation tables
admit at most 12 columns and one 20pt text line per emitted row cell, matching the writer's fixed
36pt row height. Longer logical rows continue through additional rows/slides; excessively wide
tables fail explicitly instead of producing unreadable output. Other document targets retain the
64-column content limit.

Presentation layout does not preserve all Note typography, merged-cell geometry, original PDF
page geometry, animations or templates. Content is retained within this explicit reflow contract.
Word/Excel to PPTX is not offered. Legacy PPT upgrade uses the vendor IR without applying the
content-authoring pagination policy.

## Bounds and validation

Sources are limited to 128 MiB. Office archive admission enforces the existing extraction ZIP limits:
20,000 entries, 128 MiB per expanded entry and 256 MiB total expansion. Semantic content is bounded
to 4 MiB, 2,000 sections/pages, 64 nesting levels, 200,000 elements/events and 64 table columns.
Individual image snapshots/normalized images are at most 32 MiB and 24 megapixels; aggregate
conversion image bytes are bounded to 64 MiB. Unsupported control/XML characters fail validation.
The private worker request is at most 16 MiB, response header 16 KiB and output 64 MiB.

Office semantic conversion validates supported content, embedded images and links before writing.
Vector drawings, hidden sections, missing images and unsupported representations fail rather than
being silently omitted. Spreadsheet content export retains displayed cell content, not workbook
behavior/formulas. Office-to-PDF is rejected in the menu, Action admission and worker; it never
silently becomes a reflowed PDF.

## Verification

Focused checks cover source identity, strict worker frames, images and changed-resource rejection,
Note structure/marks/merged tables, literal recognition, PPTX content retention/pagination, action
admission, failure causes and process lifecycle. Execute after the development changes are complete:

```sh
pnpm check:web
pnpm check:rust
cargo test --locked --manifest-path src-tauri/Cargo.toml documents::generation
cargo test --locked --manifest-path src-tauri/Cargo.toml spaces::derivation::tests::document_formats
```

Open representative Word/PDF/PowerPoint outputs and inspect rendered pages. Binary signatures,
XML assertions and passing unit tests do not establish visual fidelity or pagination quality.
Source selection evidence belongs to [oss-first.md](oss-first.md). Native PDF image rendering and
format output require acceptance on each supported platform before release.
