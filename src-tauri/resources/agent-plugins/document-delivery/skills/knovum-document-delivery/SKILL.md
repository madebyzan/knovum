---
name: knovum-document-delivery
description: Produce a source-grounded document and deliver a requested DOCX, PDF, TXT, or presentation artifact using Knovum's real draft, export, and render Actions.
---

# Document delivery

Use this skill when the requested outcome is a report, proposal, handout, or presentation that must be saved or exported. Respond in the user's language. The exported artifact, not an outline or a private workspace draft, is the deliverable when execution is authorized.

## Prepare the document

1. Determine the audience, purpose, output format, and material constraints from the request. Ask about format only when the intended deliverable cannot be inferred. Do not ask the user to approve routine reversible work already authorized.
2. Read authorized source nodes and their current revisions. Draft an outline around the reader's decision or task, then gather evidence for the substantive claims. Cite the source locations and label assumptions or missing evidence.
3. Reuse a suitable existing document when the user asks to export it. Do not regenerate an already satisfactory document or create a duplicate merely to use this skill.
4. When a new document is requested in Execute mode, the built-in Agent uses the admitted `draft-document` Action with the available schema, a concrete title and a purpose covering audience, structure, sources, and required output. An external Agent drafts and generates the requested final artifact with its own authenticated runtime and available document tools; Knovum inference Actions are unavailable to it. Inspect the document before delivery. Private files do not count as published nodes.

## Deliver the requested artifact

For an existing published document, use its current revision and advertised deterministic Action contract. These are business Action IDs, not literal tool names:

- DOCX: `export-document-as-docx`.
- PDF: `export-document-as-pdf`.
- Plain text: `export-document-as-txt`.
- Presentation: `render-presentation`.

Submit only the requested format unless the user asks for alternatives. A presentation should be organized for slides: one main point per slide, concise supporting content, and evidence references. Use the renderer's actual document structure rather than pretending arbitrary Markdown is a finished slide deck.

An external Agent creating a new deliverable writes the final file in the requested format to the current task output directory; Knovum validates and publishes task outputs after execution ends. Keep intermediates in hidden directories. A Markdown draft is not a completed PDF or DOCX export, and cannot substitute for the requested format. Use only tools actually available to the executor.

For Ask or Plan, describe the document and proposed delivery without publishing. For Execute, follow the admitted Run to its real terminal state. If a model, input, renderer, or authorization is missing, identify that concrete blocker; do not claim a successful export or create an empty substitute. Report the published artifact node, format, and completed Action or task output receipt, or accurately identify a file awaiting publication. Do not claim visual review of an artifact you could not inspect.

## On-demand resources

Read `references/delivery-review.md` when its review criteria apply. Use `templates/deliverable-outline.md` when a structured working record helps this task. Load only the resource needed for the current step; adapt its structure and omit unused sections.
