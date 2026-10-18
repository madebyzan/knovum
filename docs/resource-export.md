# Resource export

This document owns how a resource leaves Knovum: the formats each resource admits, the content
guarantees of each serialization and renderer acceptance. Spaces owns source authorization,
revisions and destination authorization; packaging owns runtime distribution.

## One export, chosen in the bar and named by its destination

Saving the same content in another format produces no new knowledge, so an export is not a
derivation and never places a second resource on the canvas. Export is two steps: the node bar
offers the formats the selection admits, and the chosen format is the whole format instruction. The
native save dialog then only names the file; a destination that does not carry the chosen extension
is given it.

`list_export_formats` answers what one resource admits, own format first, and is the only place
that decides. It answers from the content, not only from the kind: every serialization reads the
same authoring model, so a document outside the authoring contract offers no format at all and
reports why. The bar asks when the formats are looked at and shows that reason where it was asked,
instead of accepting a format and failing after a destination was picked. A source file's own bytes
are admitted by extension and checked when they are read. A section is an arrangement rather than a
resource, so its one format is the outline of what it holds.
`export_resource` writes the destination the dialog returned: replacing an existing
file is what the person already confirmed, a failed or cancelled run leaves no partial file, and the
source revision is checked again afterwards so a resource that changed mid-export is never delivered
as the current one. Every export is addressable by job identity and can be stopped by the person who
started it. Formats the Renderer serializes from content it already holds - Markdown, a section
outline, an object or a credential summary - are written through `export_text` instead; the engine
truthfully reports no format for them rather than failing, and they stay available even when the
engine admits nothing.

## Supported formats

| Resource           | Formats, own format first                       | Contract                                                                          |
| ------------------ | ----------------------------------------------- | --------------------------------------------------------------------------------- |
| Rich Note, text    | TXT, HTML, DOCX, PDF, PPTX                      | Structured authoring content; no Markdown intermediate                            |
| Markdown           | MD, TXT, HTML, DOCX, PDF, PPTX                  | Semantic content and authorized managed images; MD is written by the Renderer     |
| PDF                | PDF, TXT, HTML, DOCX, PPTX                      | Editable text reflow, source-page images; qualified recognition for scanned pages |
| DOC/DOCX, XLS/XLSX | own format, TXT, HTML, DOCX, PDF                | Supported semantic Office content, reflowed rather than laid out as the original  |
| PPT/PPTX           | own format, TXT, HTML, DOCX, PDF; PPT adds PPTX | Supported Office IR content and legacy upgrade                                    |
| Other file         | own format                                      | The managed bytes as they are                                                     |
| Image              | own format when untouched, PNG, JPG             | The picture as it is currently edited; JPEG composites transparency onto white    |
| Relative depth     | own format                                      | Measurement data, copied verbatim rather than re-encoded as a picture             |
| Audio              | own container, WAV, MP3, M4A, FLAC              | The same recording, re-encoded by the qualified bundled FFmpeg encoders           |
| Video              | own container, MP4, MOV                         | H.264 and AAC, the pair every player and editor admits                            |

A resource that is still being prepared reports that it is unfinished instead of offering a format.
An extension the resource does not admit is refused in Rust; lossless WebP is neither small nor
lossy, so it is not offered as an image target. `media-libraries` qualifies every encoder named here,
so a supplier that lacks one fails the build rather than the export.

The private document worker owns synchronous parsing and serialization. Markdown is parsed with
`pulldown-cmark`; Notes are translated directly from their validated Tiptap document into the admitted
`office_oxide` IR. PDF text is freshly extracted from the pinned original file, with revision-qualified
recognition supplied as a separate literal page input. Knovum keeps source/asset policy, format
translation, presentation pagination and safe typesetting.

Original-layout fidelity is outside this product boundary. Knovum does not install, bundle or
implicitly discover an Office layout engine; every document source is read into the authoring model
and written by the same writers, so one reading reaches every format a document can become. Office
content therefore reflows - PDF included - and claims no original pagination or positioning. Users
who need the original layout open the file in their external Office application. No second
persistent document model, universal conversion service, compatibility reader or alternate engine
after failure is introduced.

The export cancellation token owns cancellation. The host kills and reaps the worker process tree
before releasing execution capacity. Source files and attached image snapshots are checked again
before the destination is written. The private worker protocol is strict v2; v1, unknown fields,
wrong frame lengths, trailing bytes and unsuccessful exits fail.

One published current source is required. Unsupported sources and processing/failed resources are
rejected in Rust. Known scanned PDF pages remain blocked until every required page has completed
recognition with nonempty text from the same source revision. Export never triggers an implicit AI
request. The host validates the recognition manifest; the worker validates required page identities
against the original PDF. OCR text is written literally, not interpreted as Markdown or executable
content.

For text-bearing PDFs, original image regions are rendered with the existing native PDF adapter and
included as figures associated with their source page. Recognized scanned pages retain a raster of
the source page alongside editable recognized text. This is content delivery, not recovery of the
original editable Word layout. Page geometry and precise reading-order placement are not promised.
TXT carries semantic text and image alternative text without exporting private paths.

File display titles name the exported document and HTML metadata. They do not become new body
headings. Markdown and Note title/body handling avoids duplicating a matching leading H1. Citations
are appended as literal structured paragraphs.

## Markdown and Notes

Markdown supports CommonMark paragraphs, H1–H6 headings, quotes, rules, line breaks, links, inline
and block code, nested ordered/unordered lists, tables, strikethrough and task lists. Parser upgrades
must not expand the contract implicitly through `Options::all()`. Math and footnote extensions remain
outside the Markdown authoring contract; custom templates are not an export parameter.

Markdown carries HTML, so a document is the content that markup wraps rather than the markup: text
belongs to the document, `<br>` is a line break, an `<img>` reads as what it shows, and a tag that
says nothing about content contributes nothing. Elements whose text is not prose - scripts, styles,
embedded markup languages - and comments contribute nothing at all. Content outside the authoring
contract is still unavailable in every format alike, and that is reported when the formats are
listed rather than when the file is written.

A text block is the same editable content as a Rich Note, presented without prose styling, so it
reaches the same formats through the same reading.

Rich Notes retain headings, aligned paragraphs, lists and checked task state, code blocks, quotes,
rules, tables including merged cells in DOCX/PDF/HTML, and images. Supported text marks include
bold, italic, strike, underline, code, subscript/superscript, highlight and the editor's text styles.
Unsupported meaningful nodes/marks fail explicitly. ReasoningTrace is editor decoration state and
is not part of the saved authoring document. Structured data is translated directly to vendor IR.

Image resources are authorized by Spaces and revision-pinned before rendering. Markdown image
URLs may resolve through their stored editor asset identity, or an absolute/relative path within the
managed Space asset directory. Remote images are not downloaded; external/private file access is
not authorized by conversion. A conversion embeds bytes, so it embeds only the pictures this Space
holds: a reference it cannot read - a badge served over the network, a path outside the Space, a
file that is gone - reads as its alternative text instead of failing the document. TXT uses
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
behavior/formulas. Office-to-PDF is never offered and is rejected by the worker; it never
silently becomes a reflowed PDF.

## Verification

Focused checks cover the admitted formats of every resource kind, source identity, strict worker
frames, images and changed-resource rejection, Note structure/marks/merged tables, literal
recognition, PPTX content retention/pagination, failure causes and process lifecycle. Execute after the development changes are complete:

```sh
pnpm check:web
pnpm check:rust
cargo test --locked --manifest-path src-tauri/Cargo.toml documents::generation
cargo test --locked --manifest-path src-tauri/Cargo.toml spaces::assets::export
```

Open representative Word/PDF/PowerPoint, image, audio and video outputs and inspect them. Binary signatures,
XML assertions and passing unit tests do not establish visual fidelity or pagination quality.
Source selection evidence belongs to [oss-first.md](oss-first.md). Native PDF image rendering and
format output require acceptance on each supported platform before release.
