# Creation sources

How a Space resource reaches a model when the user asks for something new. One document for the
whole path: what the user selects, what the model receives, and who decides between the two.

## The rule

> Every node on the canvas is knowledge. Anything a user can select, they can create from.

There are four things a user creates: a **document**, an **image**, a **video**, an **audio**
reading. Everything else is a detail of how that happens. There is no separate "edit an image"
request: attaching pixels and asking for a change is the same request as asking for a new image.

Availability is a property of the **Action and the file type**, never of an asynchronous background
job. "This image has not been analyzed yet" is a reason to prepare during the Run, not a reason to
disable a menu entry.

## Three forms a source can take

A selected node enters a model in one of three forms. The form is chosen by capability negotiation,
not by the UI:

| Form            | What travels                                    | When it is chosen                                  |
| --------------- | ----------------------------------------------- | -------------------------------------------------- |
| **Original**    | the actual bytes (rendered PNG, document file)  | the target accepts binary input of that media type |
| **Text**        | extracted or indexed text, retrieved when large | the source is textual, or the target is text-only  |
| **Description** | cached visual/media analysis                    | fallback only: the target cannot read the original |

Preference order is Original → Text → Description. A description is a lossy substitute for pixels
the model could have seen; it is never preferred over the original, and it is never a precondition.

### Why this order matters

Sending an image through its own description costs an extra model call, waits for a background job,
and loses everything the description did not mention. Sending the pixels costs nothing extra and
preserves identity, composition, palette and text in the image — which is the whole point of
"make me a variant of these".

## Pinning a selection

Each selected resource is pinned to the revision the user is looking at, on the basis that made it
readable:

| Basis       | Pinned value        | Used for                                          |
| ----------- | ------------------- | ------------------------------------------------- |
| `asset`     | the file's bytes    | images, which are readable as pixels on their own |
| `knowledge` | the indexed content | text, documents, websites, transcripts            |

The basis is decided once, where the selection is resolved, and then travels with the input through
the Run and into the derivation record. No handler assumes a basis of its own, which is what used to
make an unanalyzed image look unusable.

## Capability negotiation

An image-generation adapter declares what it accepts instead of the shared layer guessing: how many
reference images, and how many bytes each may be. The shared layer refuses what the adapter did not
declare, and the caller renders within the declared budget instead of discovering a provider limit
through a failed request. No adapter silently drops an attachment.

Verified anchors for the adapters that accept references today:

- Gemini image models accept reference images per model (3 for the 2.5 Flash image model, up to 14
  for the 3.x image models), around 7 MB per file.
- The OpenAI-family shapes (Grok edits, Doubao Seedream, OpenRouter image references, SiliconFlow
  Qwen-Image-Edit), DashScope's qwen-image protocol and Kling's omni-image models each accept one
  image alongside the prompt.

A model with no declared reference support is a text-only image model: its sources fall back to
Text, and the creation entry stays available.

### Transport: a file reference beats base64

Where a provider has a file API the pixels are uploaded and the request carries the reference —
Gemini `fileData`, Grok `file_id`, DashScope's uploaded URL. Inline base64 is used only where the
API offers nothing else (Doubao, OpenRouter, SiliconFlow, Kling), because it inflates the payload by
about a third. The choice belongs to the adapter, never to the calling feature.

## Preparing a source

Before bytes leave the process, every source goes through one shared preparation:

1. render the current pixels, applying the node's pending image edit
2. verify the content revision again, so a queued Run cannot send an image the user changed
3. check the media type against what the renderer accepts
4. downscale when a file exceeds the declared budget, rather than failing the Run
5. own the temporary file until the request completes

Rendering happens outside the write transaction: it is CPU work, and its correctness rests on the
content revision check, not on a database lock.

## What the user sees

One creation dialog: what you need, in words; the output switch (document / image / video); the few
options that output actually has; the executor. The selected resources are listed as the material.
There is no style picker and no per-output dialog — the requirement text says what the style is.

When a source still needs preparation, the Run says so as a stage on the output node that was
already reserved when the user clicked. A disabled menu entry is reserved for the cases that cannot
be fixed by waiting: a selection larger than a Run can pin, or a selection with nothing a model can
read, each stated plainly.

The same rule covers content that arrives from outside. A pasted media link already says what it
will become - a link to a page is saved as a website, and a link to media says whether it is audio
or video - so its node is reserved the moment the link is dropped, before anything is downloaded.
Arriving media is ordinary canvas content from the first moment: it can be moved, selected and left
alone while it finishes, and it survives a restart. The import is asked for the kind the link
promised and must publish into the asset the node holds, so the node the canvas has been showing is
the node that settles. Work names the node it belongs to, so the node shows the stage it is in and
offers to stop it, while the Task Center carries the same work as a Run. An output's `derives-from`
Relations are written with the node, not when it settles, so lineage is visible for the whole time
the work takes rather than appearing at the end. Nothing is left behind: an import that fails or is
cancelled takes its reserved node and its asset away.

## Settling

A derived node settles onto an asset the Space itself published, and it reaches the same standing
an imported node has. It carries its own cover, made from the file that was just published rather
than borrowed from its source: the picture is new even when its subject is not, and the geometry
that cover reports is what gives a rendered video its aspect ratio on the canvas. A cover made for
an output that then fails to publish leaves with its file. What its new asset still has to say
about itself is then scheduled exactly as an import's is, and the payload decides what that is.

## Inheritance

What a derived node keeps of its source is a property of the transform, so it is declared once,
for every output made from one source, in `spaces::derivation::inheritance` — not decided by
whichever code happens to publish it. Everything the published file establishes (path, revision,
size, cover, geometry) is always the output's own. What can be kept is knowledge about the content:

| Facet    | Field             |
| -------- | ----------------- |
| Labels   | `tags`            |
| Timeline | `durationSeconds` |
| Sound    | `mediaAnalysis`   |
| Speech   | `mediaKnowledge`  |
| Picture  | `visualKnowledge` |

| Output                                                                 | Keeps                                    |
| ---------------------------------------------------------------------- | ---------------------------------------- |
| Subtitled video                                                        | Labels, Timeline, Sound, Speech, Picture |
| Dubbed video                                                           | Labels, Picture                          |
| Edited video                                                           | Labels                                   |
| Separated vocals                                                       | Speech                                   |
| Upscaled image                                                         | Labels, Picture                          |
| Background or object removed                                           | Labels                                   |
| Separated background, extracted object, depth map, new view, narration | nothing                                  |

Kept knowledge is re-pinned to the published asset, which is what tells enrichment it has nothing
to read again: adding subtitles schedules no work at all. Knowledge that is not kept is absent,
which is what asks enrichment to establish the output's own — a dubbed video is measured and
transcribed afresh, an edit re-reads its new timeline. Only knowledge that is current for the source
is kept; a stale reading is not passed off as describing a new file. Labels a person gave the
output while it was being made stand.

Some things are never kept. Provenance (`origin`, `creator`, `externalSource`) is the source's own,
and what explains a derived node is its Relations and its Action record. Annotations (`comments`,
`imageEdit`, `videoEditProject`) belong to the node they were made on; an edit is already rendered
into whatever was derived from it. Outputs made from knowledge rather than one asset — generated
images, videos and documents, visualizations, the layers a picture is split into — are new works.

The reservation enforces this: a reserved output holds only its name, the path and size of the
asset it will publish, and labels. A reservation that already carries knowledge has taken it from
somewhere it should not, and it cannot publish.

## Reuse

A derived node records its Action, pinned parameters, sources and execution identity. That is
everything a repeat needs, so reuse reads the result rather than capturing a workflow in advance.
